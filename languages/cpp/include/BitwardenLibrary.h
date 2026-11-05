#pragma once

#include <string>

#ifdef _WIN32
#include <windows.h>
#else
#include <dlfcn.h>
#endif

class BitwardenLibrary {
public:
    BitwardenLibrary(const std::string& providedLibraryPath);
    ~BitwardenLibrary();

    void* init(const char* clientSettingsJson);
    void free_mem(void* client);
    const char* run_command(const char* commandJson, void* client);
    void free_string(const char* str);

private:
#ifdef _WIN32
    HMODULE libraryHandle;
#else
    void* libraryHandle;
#endif
};

