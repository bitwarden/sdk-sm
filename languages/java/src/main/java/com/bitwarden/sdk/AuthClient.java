package com.bitwarden.sdk;

import com.bitwarden.sdk.schema.*;

import java.util.function.Function;

public class AuthClient {

    private final CommandRunner commandRunner;

    AuthClient(CommandRunner commandRunner) {
        this.commandRunner = commandRunner;
    }

    public AccessTokenLoginResponse loginAccessToken(String accessToken, String stateFile) {
        Command command = new Command();
        AccessTokenLoginRequest accessTokenLoginRequest = new AccessTokenLoginRequest();
        accessTokenLoginRequest.setAccessToken(accessToken);
        accessTokenLoginRequest.setStateFile(stateFile);

        command.setLoginAccessToken(accessTokenLoginRequest);

        ResponseForAccessTokenLoginResponse response = commandRunner.runCommand(command,
            BitwardenClient.throwingFunctionWrapper(Converter::ResponseForAccessTokenLoginResponseFromJsonString));

        if (response == null || !response.getSuccess()) {
            throw new BitwardenClientException(response != null ? response.getErrorMessage() : "Login failed");
        }

        return response.getData();
    }
}
