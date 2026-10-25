#pragma once

#include <string>
#include <functional>
#include "BitwardenLibrary.h"
#include "schemas.hpp"
#include <iostream>

using namespace Bitwarden::Sdk;

class CommandRunner {
public:
    CommandRunner(BitwardenLibrary* library, void* client);

    template <typename T, typename R, typename Func>
    R runCommand(const Command& command, Func deserializer);



private:
    BitwardenLibrary* library;
    void* client;

    std::string commandToString(const Command& command);
    nlohmann::json filterNullObjects(const nlohmann::json& input);
};

template <typename T, typename R, typename Func>
R CommandRunner::runCommand(const Command& command, Func deserializer) {
    // Serialize the Command object to a JSON string
    std::string jsonString = commandToString(command);
    const char* jsonCStr = jsonString.c_str();
    const char* rawResponse = library->run_command(jsonCStr, client);
    if (rawResponse == nullptr) {
        throw std::runtime_error("Failed to execute command: run_command returned null");
    }
    std::string response(rawResponse);
    library->free_string(rawResponse);

    // Deserialize the response using the provided deserializer function
    T deserialized = deserializer(response.c_str());

    // Unwrap the response and throw an exception if it was not successful
    if (!deserialized.get_success()) {
        throw std::runtime_error(*deserialized.get_error_message());
    }

    return deserialized.get_data().get();
}

