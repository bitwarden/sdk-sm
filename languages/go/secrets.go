package sdk

import (
	"fmt"
	"time"
)

type SecretsInterface interface {
	Create(key, value, note string, projectIDs []string) (*SecretResponse, error)
	List() (*SecretIdentifiersResponse, error)
	Get(secretID string) (*SecretResponse, error)
	GetByIDS(secretIDs []string) (*SecretsResponse, error)
	Update(secretID string, key, value, note string, projectIDs []string) (*SecretResponse, error)
	Delete(secretIDs []string) (*SecretsDeleteResponse, error)
	Sync(lastSyncedDate *time.Time) (*SecretsSyncResponse, error)
}

type Secrets struct {
	CommandRunner CommandRunnerInterface
	Client        BitwardenClientInterface
}

func NewSecrets(commandRunner CommandRunnerInterface, client BitwardenClientInterface) *Secrets {
	return &Secrets{CommandRunner: commandRunner, Client: client}
}

func (s *Secrets) executeCommand(command Command, target any) error {
	responseStr, err := s.CommandRunner.RunCommand(command)
	if err != nil {
		return err
	}
	return checkSuccessAndError(responseStr, target)
}

func (s *Secrets) Create(key, value, note string, projectIDs []string) (*SecretResponse, error) {
	orgID, err := s.Client.GetAccessTokenOrganization()
	if err != nil {
		return nil, err
	}
	if orgID == "" {
		return nil, fmt.Errorf("no organization found in access token")
	}

	command := Command{
		Secrets: &SecretsCommand{
			Create: &SecretCreateRequest{
				Key:            key,
				Value:          value,
				Note:           note,
				OrganizationID: orgID,
				ProjectIDS:     projectIDs,
			},
		},
	}

	var response SecretResponse
	if err := s.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (s *Secrets) List() (*SecretIdentifiersResponse, error) {
	orgID, err := s.Client.GetAccessTokenOrganization()
	if err != nil {
		return nil, err
	}
	if orgID == "" {
		return nil, fmt.Errorf("no organization found in access token")
	}

	command := Command{
		Secrets: &SecretsCommand{
			List: &SecretIdentifiersRequest{
				OrganizationID: orgID,
			},
		},
	}

	var response SecretIdentifiersResponse
	if err := s.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (s *Secrets) Get(id string) (*SecretResponse, error) {
	command := Command{
		Secrets: &SecretsCommand{
			Get: &SecretGetRequest{
				ID: id,
			},
		},
	}

	var response SecretResponse
	if err := s.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (s *Secrets) GetByIDS(ids []string) (*SecretsResponse, error) {
	command := Command{
		Secrets: &SecretsCommand{
			GetByIDS: &SecretsGetRequest{
				IDS: ids,
			},
		},
	}

	var response SecretsResponse
	if err := s.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (s *Secrets) Update(id string, key, value, note string, projectIDs []string) (*SecretResponse, error) {
	orgID, err := s.Client.GetAccessTokenOrganization()
	if err != nil {
		return nil, err
	}
	if orgID == "" {
		return nil, fmt.Errorf("no organization found in access token")
	}

	command := Command{
		Secrets: &SecretsCommand{
			Update: &SecretPutRequest{
				ID:             id,
				Key:            key,
				Value:          value,
				Note:           note,
				OrganizationID: orgID,
				ProjectIDS:     projectIDs,
			},
		},
	}

	var response SecretResponse
	if err := s.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (s *Secrets) Delete(ids []string) (*SecretsDeleteResponse, error) {
	command := Command{
		Secrets: &SecretsCommand{
			Delete: &SecretsDeleteRequest{
				IDS: ids,
			},
		},
	}

	var response SecretsDeleteResponse
	if err := s.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (s *Secrets) Sync(lastSyncedDate *time.Time) (*SecretsSyncResponse, error) {
	orgID, err := s.Client.GetAccessTokenOrganization()
	if err != nil {
		return nil, err
	}
	if orgID == "" {
		return nil, fmt.Errorf("no organization found in access token")
	}

	command := Command{
		Secrets: &SecretsCommand{
			Sync: &SecretsSyncRequest{
				OrganizationID: orgID,
				LastSyncedDate: lastSyncedDate,
			},
		},
	}

	var response SecretsSyncResponse
	if err := s.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}
