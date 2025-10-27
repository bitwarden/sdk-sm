<?php

namespace Bitwarden\Sdk;

use Bitwarden\Sdk\Schemas\ClientSettings;
use Bitwarden\Sdk\Schemas\Command;
use Exception;
use FFI;
use JsonException;
use RuntimeException;
use stdClass;

class BitwardenLib
{
    public FFI $ffi;
    public FFI\CData $handle;

    /**
     * @throws Exception
     */
    public function __construct()
    {
        $lib_file = null;

        if (PHP_OS === 'WINNT') {
            $lib_file = __DIR__ . '/lib/windows-x64/bitwarden_c.dll';
            if (!file_exists($lib_file)) {
                $lib_file = __DIR__ . '/../../../target/debug/bitwarden_c.dll';
            }
        } elseif (PHP_OS === 'Linux') {
            $lib_file = __DIR__ . '/lib/linux-x64/libbitwarden_c.so';
            if (!file_exists($lib_file)) {
                $lib_file = __DIR__ . '/../../../target/debug/libbitwarden_c.so';
            }
        } elseif (PHP_OS === 'Darwin') {
            $architecture = php_uname('m');
            if ($architecture === 'x86_64' || $architecture === 'amd64') {
                $lib_file = __DIR__ . '/lib/macos-x64/libbitwarden_c.dylib';
            } elseif ($architecture === 'arm64') {
                $lib_file = __DIR__ . '/lib/macos-arm64/libbitwarden_c.dylib';
            }
            if (!file_exists($lib_file)) {
                $lib_file = __DIR__ . '/../../../target/debug/libbitwarden_c.dylib';
            }
        }

        if ($lib_file == null || !is_file($lib_file)) {
            throw new Exception("Lib file not found");
        }

        $this->ffi = FFI::cdef('
            void* init(const char* param);
            char* run_command(void* c_str_ptr, void* client_ptr);
            void free_mem(void* client_ptr);',
            $lib_file
        );
    }

    /**
     * @throws JsonException
     * @throws Exception
     */
    public function init(ClientSettings $client_settings): FFI\CData
    {
        $encoded_json = $this::json_encode_sdk_format($client_settings->to());
        $this->handle = $this->ffi->init($encoded_json);
        return $this->handle;
    }

    /**
     * @throws JsonException
     * @throws Exception
     */
    public function run_command(Command $command): stdClass
    {
        $encoded_json = $this::json_encode_sdk_format($command->to());
        try {
            $result = $this->ffi->run_command($encoded_json, $this->handle);
            return json_decode(FFI::string($result));
        } catch (FFI\Exception $e) {
            throw new RuntimeException('Error occurred during FFI operation: ' . $e->getMessage());
        }
    }

    public function free_mem(): void
    {
        $this->ffi->free_mem($this->handle);
    }

    /**
     * @throws JsonException
     */
    private static function json_encode_sdk_format($object): string
    {
        $withoutNull = function ($a) use (&$withoutNull) {
            if (is_object($a)) {
                $a = array_filter((array)$a);
                foreach ($a as $k => $v) {
                    $a[$k] = $withoutNull($v);
                }

                return (object)$a;
            }

            return $a;
        };

        $object_no_nulls = $withoutNull($object);

        return json_encode($object_no_nulls, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    }
}
