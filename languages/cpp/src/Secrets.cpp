#include "Secrets.h"
#include <nlohmann/json.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <boost/optional.hpp>
#include <sstream>

Secrets::Secrets(CommandRunner* commandRunner) : commandRunner(commandRunner) {}

auto secretsDeserializer = [](const std::string& response) -> ResponseForSecretResponse {
    nlohmann::json jsonResponse = nlohmann::json::parse(response);
    ResponseForSecretResponse secretResponse;
    Bitwarden::Sdk::from_json(jsonResponse, secretResponse);
    return secretResponse;
};

auto secretsByIdsDeserializer = [](const std::string& response) -> ResponseForSecretsResponse {
    nlohmann::json jsonResponse = nlohmann::json::parse(response);
    ResponseForSecretsResponse secretsResponse;
    Bitwarden::Sdk::from_json(jsonResponse, secretsResponse);
    return secretsResponse;
};

auto deleteSecretsDeserializer = [](const std::string& response) -> ResponseForSecretsDeleteResponse {
    nlohmann::json jsonResponse = nlohmann::json::parse(response);
    ResponseForSecretsDeleteResponse deleteSecretsResponse;
    Bitwarden::Sdk::from_json(jsonResponse, deleteSecretsResponse);
    return deleteSecretsResponse;
};

auto secretListDeserializer = [](const std::string& response) -> ResponseForSecretIdentifiersResponse {
    nlohmann::json jsonResponse = nlohmann::json::parse(response);
    ResponseForSecretIdentifiersResponse listResponse;
    Bitwarden::Sdk::from_json(jsonResponse, listResponse);
    return listResponse;
};

auto secretsSyncDeserializer = [](const std::string& response) -> ResponseForSecretsSyncResponse {
    nlohmann::json jsonResponse = nlohmann::json::parse(response);
    ResponseForSecretsSyncResponse syncResponse;
    Bitwarden::Sdk::from_json(jsonResponse, syncResponse);
    return syncResponse;
};

SecretResponse Secrets::get(const boost::uuids::uuid& id) {
    Command command;
    SecretsCommand secretsCommand;
    SecretGetRequest secretGetRequest;

    std::string idStr = boost::uuids::to_string(id);
    secretGetRequest.set_id(idStr);

    secretsCommand.set_get(secretGetRequest);
    command.set_secrets(secretsCommand);

    try {
        return commandRunner->runCommand<ResponseForSecretResponse, SecretResponse>(command, secretsDeserializer);
    } catch (const std::exception& ex) {
        std::cerr << "Error in getSecret: " << ex.what() << std::endl;
        throw ex;
    }
}

SecretsResponse Secrets::getByIds(const std::vector<boost::uuids::uuid>& ids) {
    Command command;
    SecretsCommand secretsCommand;
    SecretsGetRequest secretsGetRequest;

    std::vector<std::string> idsStr;
    for (const auto& id : ids) {
        idsStr.push_back(boost::uuids::to_string(id));
    }
    secretsGetRequest.set_ids(idsStr);

    secretsCommand.set_get_by_ids(secretsGetRequest);
    command.set_secrets(secretsCommand);

    try {
        return commandRunner->runCommand<ResponseForSecretsResponse, SecretsResponse>(command, secretsByIdsDeserializer);
    } catch (const std::exception& ex) {
        std::cerr << "Error in getSecretsByIds: " << ex.what() << std::endl;
        throw ex;
    }
}

SecretResponse Secrets::create(const boost::uuids::uuid& organizationId, const std::string& key, const std::string& value, const std::string& note, const std::vector<boost::uuids::uuid>& projectIds) {
    Command command;
    SecretsCommand secretsCommand;
    SecretCreateRequest secretCreateRequest;

    std::string orgIdStr = boost::uuids::to_string(organizationId);
    secretCreateRequest.set_organization_id(orgIdStr);

    secretCreateRequest.set_key(key);
    secretCreateRequest.set_value(value);
    secretCreateRequest.set_note(note);

    std::vector<std::string> projectIdsStr;
    for (const auto& projectId : projectIds) {
        projectIdsStr.push_back(boost::uuids::to_string(projectId));
    }
    secretCreateRequest.set_project_ids(projectIdsStr);

    secretsCommand.set_create(secretCreateRequest);
    command.set_secrets(secretsCommand);

    try {
        return commandRunner->runCommand<ResponseForSecretResponse, SecretResponse>(command, secretsDeserializer);
    } catch (const std::exception& ex) {
        std::cerr << "Error in createSecret: " << ex.what() << std::endl;
        throw ex;
    }
}

SecretResponse Secrets::update(const boost::uuids::uuid& organizationId, const boost::uuids::uuid& id, const std::string& key, const std::string& value, const std::string& note, const std::vector<boost::uuids::uuid>& projectIds) {
    Command command;
    SecretsCommand secretsCommand;
    SecretPutRequest secretPutRequest;

    std::string idStr = boost::uuids::to_string(id);
    secretPutRequest.set_id(idStr);

    std::string orgIdStr = boost::uuids::to_string(organizationId);
    secretPutRequest.set_organization_id(orgIdStr);

    secretPutRequest.set_key(key);
    secretPutRequest.set_value(value);
    secretPutRequest.set_note(note);

    std::vector<std::string> projectIdsStr;
    for (const auto& projectId : projectIds) {
        projectIdsStr.push_back(boost::uuids::to_string(projectId));
    }
    secretPutRequest.set_project_ids(projectIdsStr);

    secretsCommand.set_update(secretPutRequest);
    command.set_secrets(secretsCommand);

    try {
        return commandRunner->runCommand<ResponseForSecretResponse, SecretResponse>(command, secretsDeserializer);
    } catch (const std::exception& ex) {
        std::cerr << "Error in updateSecret: " << ex.what() << std::endl;
        throw ex;
    }
}

SecretsDeleteResponse Secrets::deleteSecrets(const std::vector<boost::uuids::uuid>& ids) {
    Command command;
    SecretsCommand secretsCommand;
    SecretsDeleteRequest secretsDeleteRequest;

    std::vector<std::string> idsStr;
    for (const auto& id : ids) {
        idsStr.push_back(boost::uuids::to_string(id));
    }
    secretsDeleteRequest.set_ids(idsStr);

    secretsCommand.set_secrets_delete(secretsDeleteRequest);
    command.set_secrets(secretsCommand);

    try {
        return commandRunner->runCommand<ResponseForSecretsDeleteResponse, SecretsDeleteResponse>(command, deleteSecretsDeserializer);
    } catch (const std::exception& ex) {
        std::cerr << "Error in deleteSecrets: " << ex.what() << std::endl;
        throw ex;
    }
}

SecretIdentifiersResponse Secrets::list(const boost::uuids::uuid& organizationId) {
    Command command;
    SecretsCommand secretsCommand;
    SecretIdentifiersRequest secretIdentifiersRequest;

    std::string orgIdStr = boost::uuids::to_string(organizationId);
    secretIdentifiersRequest.set_organization_id(orgIdStr);

    secretsCommand.set_list(secretIdentifiersRequest);
    command.set_secrets(secretsCommand);

    try {
        return commandRunner->runCommand<ResponseForSecretIdentifiersResponse, SecretIdentifiersResponse>(command, secretListDeserializer);
    } catch (const std::exception& ex) {
        std::cerr << "Error in listSecret: " << ex.what() << std::endl;
        throw ex;
    }
}

SecretsSyncResponse Secrets::sync(const boost::uuids::uuid& organizationId, const boost::optional<std::chrono::system_clock::time_point>& lastSyncedDate) {
    Command command;
    SecretsCommand secretsCommand;
    SecretsSyncRequest secretsSyncRequest;

    std::string orgIdStr = boost::uuids::to_string(organizationId);
    secretsSyncRequest.set_organization_id(orgIdStr);

    if (lastSyncedDate.has_value()) {
        auto timePoint = lastSyncedDate.value();

        // Get time as time_t and milliseconds
        auto timeT = std::chrono::system_clock::to_time_t(timePoint);
        auto milliseconds = std::chrono::duration_cast<std::chrono::milliseconds>(timePoint.time_since_epoch()) % 1000;

        // Convert to a tm struct
        std::tm tm = *std::gmtime(&timeT);

        // Create a string stream to format the date and time
        std::stringstream dateStream;
        dateStream << std::put_time(&tm, "%Y-%m-%dT%H:%M:%S");

        // Add milliseconds
        dateStream << '.' << std::setw(3) << std::setfill('0') << milliseconds.count() << 'Z';

        // Convert to string
        std::string dateStr = dateStream.str();

        // Set the last synced date
        secretsSyncRequest.set_last_synced_date(dateStr);
    } else {
        secretsSyncRequest.set_last_synced_date(boost::none);
    }

    secretsCommand.set_sync(secretsSyncRequest);
    command.set_secrets(secretsCommand);

    try {
        return commandRunner->runCommand<ResponseForSecretsSyncResponse, SecretsSyncResponse>(command, secretsSyncDeserializer);
    } catch (const std::exception& ex) {
        std::cerr << "Error in syncSecrets: " << ex.what() << std::endl;
        throw ex;
    }
}
