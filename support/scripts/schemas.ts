import {
  quicktype,
  quicktypeMultiFile,
  InputData,
  JSONSchemaInput,
  FetchingJSONSchemaStore,
} from "quicktype-core";

import fs from "fs";

async function main() {
  const schemaInput = new JSONSchemaInput(new FetchingJSONSchemaStore());
  const inputData = new InputData();
  inputData.addInput(schemaInput);
  inputData.addSource(
    "schema",
    {
      name: "SchemaTypes",
      uris: ["support/schemas/schema_types/SchemaTypes.json#/definitions/"],
    },
    () => new JSONSchemaInput(new FetchingJSONSchemaStore()),
  );

  const ts = await quicktype({
    inputData,
    lang: "typescript",
    rendererOptions: {},
  });

  writeToFile("./languages/js/sdk-client/src/schemas.ts", ts.lines);
  writeToFile("./crates/bitwarden-napi/src-ts/bitwarden_client/schemas.ts", ts.lines);

  const python = await quicktype({
    inputData,
    lang: "python",
    rendererOptions: {
      "python-version": "3.7",
    },
  });

  writeToFile("./languages/python/bitwarden_sdk/schemas.py", python.lines);

  const ruby = await quicktype({
    inputData,
    lang: "ruby",
    rendererOptions: {
      "ruby-version": "3.0",
    },
  });

  writeToFile("./languages/ruby/bitwarden_sdk_secrets/lib/schemas.rb", ruby.lines);

  const csharp = await quicktype({
    inputData,
    lang: "csharp",
    rendererOptions: {
      namespace: "Bitwarden.Sdk",
      framework: "SystemTextJson",
      "csharp-version": "6",
    },
  });

  writeToFile("./languages/csharp/Bitwarden.Sdk/schemas.cs", csharp.lines);

  const cpp = await quicktype({
    inputData,
    lang: "cpp",
    rendererOptions: {
      namespace: "Bitwarden::Sdk",
      "include-location": "global-include",
    },
  });

  cpp.lines.forEach((line, idx) => {
    // Replace DOMAIN for URI_DOMAIN, because DOMAIN is an already defined macro
    cpp.lines[idx] = line.replace(/DOMAIN/g, "URI_DOMAIN");
  });

  writeToFile("./languages/cpp/include/schemas.hpp", cpp.lines);

  const go = await quicktype({
    inputData,
    lang: "go",
    rendererOptions: {
      package: "sdk",
      "omit-empty": true,
    },
  });

  writeToFile("./languages/go/schema.go", go.lines);

  const java = await quicktypeMultiFile({
    inputData,
    lang: "java",
    rendererOptions: {
      package: "com.bitwarden.sdk.schema",
      "java-version": "8",
    },
  });

  const javaDir = "./languages/java/src/main/java/com/bitwarden/sdk/schema/";
  if (!fs.existsSync(javaDir)) {
    fs.mkdirSync(javaDir);
  }
  java.forEach((file, path) => {
    writeToFile(javaDir + path, file.lines);
  });

  const php = await quicktype({
    inputData,
    lang: "php",
    inferUuids: false,
    inferDateTimes: false,
    rendererOptions: {
      "acronym-style": "camel",
      "with-get": false,
    },
  });

  const phpDir = "./languages/php/src/Schemas/";
  if (!fs.existsSync(phpDir)) {
    fs.mkdirSync(phpDir);
  }

  php.lines.splice(1, 0, "namespace Bitwarden\\Sdk\\Schemas;", "use stdClass;", "use Exception;");

  writeToFile("./languages/php/src/Schemas/Schemas.php", php.lines);
}

main();

function writeToFile(filename: string, lines: string[]) {
  const output = fs.createWriteStream(filename);
  lines.forEach((line) => {
    output.write(line + "\n");
  });
  output.close();
}
