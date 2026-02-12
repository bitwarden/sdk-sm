package sdk

import (
	"encoding/json"

	"github.com/bitwarden/sdk-go/v2/internal/cinterface"
)

type BitwardenClientInterface interface {
	AccessTokenLogin(accessToken string, stateFile *string) error
	Projects() ProjectsInterface
	Secrets() SecretsInterface
	Generators() GeneratorsInterface
	Close()
}

type BitwardenClient struct {
	client        cinterface.ClientPointer
	lib           cinterface.BitwardenLibrary
	commandRunner CommandRunnerInterface
	projects      ProjectsInterface
	secrets       SecretsInterface
	generators    GeneratorsInterface
}

func NewBitwardenClient(apiURL *string, identityURL *string) (BitwardenClientInterface, error) {
	deviceType := DeviceType("SDK")
	userAgent := "Bitwarden GOLANG-SDK"
	clientSettings := ClientSettings{
		APIURL:      apiURL,
		IdentityURL: identityURL,
		UserAgent:   &userAgent,
		DeviceType:  &deviceType,
	}

	settingsJSON, err := json.Marshal(clientSettings)
	if err != nil {
		return nil, err
	}

	lib := cinterface.NewBitwardenLibrary()
	client, err := lib.Init(string(settingsJSON))
	if err != nil {
		return nil, err
	}
	runner := NewCommandRunner(client, lib)

	return &BitwardenClient{
		lib:           lib,
		client:        client,
		commandRunner: runner,
		projects:      NewProjects(runner),
		secrets:       NewSecrets(runner),
		generators:    NewGenerators(runner),
	}, nil
}

func (c *BitwardenClient) AccessTokenLogin(accessToken string, stateFile *string) error {
	req := AccessTokenLoginRequest{AccessToken: accessToken, StateFile: stateFile}
	command := Command{LoginAccessToken: &req}

	responseStr, err := c.commandRunner.RunCommand(command)
	if err != nil {
		return err
	}

	var response AccessTokenLoginResponse
	return checkSuccessAndError(responseStr, &response)
}

func (c *BitwardenClient) Projects() ProjectsInterface {
	return c.projects
}

func (c *BitwardenClient) Secrets() SecretsInterface {
	return c.secrets
}

func (c *BitwardenClient) Generators() GeneratorsInterface {
	return c.generators
}

func (c *BitwardenClient) Close() {
	c.lib.FreeMem(c.client)
}
