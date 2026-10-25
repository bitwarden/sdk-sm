package com.bitwarden.sdk;

import com.bitwarden.sdk.schema.Command;
import com.bitwarden.sdk.schema.Converter;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.jna.Pointer;

import java.io.IOException;
import java.util.function.Function;

class CommandRunner {

    private final BitwardenLibrary library;

    private final Pointer client;

    CommandRunner(BitwardenLibrary library, Pointer client) {
        this.library = library;
        this.client = client;
    }

    <T> T runCommand(Command command, Function<String, T> deserializer) {
        Pointer ptr;

        try {
            ptr = library.run_command(commandToString(command), client);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        String response;
        try {
            response = ptr.getString(0);
        } finally {
            library.free_string(ptr);
        }

        return deserializer.apply(response);
    }

    private String commandToString(Command command) throws IOException {
        // Removes null properties from the generated converter output to avoid command errors
        String inputJson = Converter.CommandToJsonString(command);

        ObjectMapper mapper = new ObjectMapper();
        mapper.setSerializationInclusion(JsonInclude.Include.NON_NULL);

        Object inputObject = mapper.readValue(inputJson, Object.class);
        return mapper.writeValueAsString(inputObject);
    }
}
