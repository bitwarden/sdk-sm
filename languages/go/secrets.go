package sdk

import (
	"fmt"
	"time"
)

type SecretsInterface interface {
	Create(key, value, note string, organizationID string, projectIDs []string) (*SecretResponse, error)
	List(organizationID string) (*SecretIdentifiersResponse, error)
	Get(secretID string) (*SecretResponse, error)
	GetByIDS(secretIDs []string) (*SecretsResponse, error)
	Update(secretID string, key, value, note string, organizationID string, projectIDs []string) (*SecretResponse, error)
	Delete(secretIDs []string) (*SecretsDeleteResponse, error)
	Sync(organizationID string, lastSyncedDate *time.Time) (*SecretsSyncResponse, error)
}

type Secrets struct {
	CommandRunner CommandRunnerInterface
}

func NewSecrets(commandRunner CommandRunnerInterface) *Secrets {
	return &Secrets{CommandRunner: commandRunner}
}

func (s *Secrets) executeCommand(command Command, target interface{}) error {
	responseStr, err := s.CommandRunner.RunCommand(command)
	if err != nil {
		return err
	}
	return checkSuccessAndError(responseStr, target)
}

func (s *Secrets) Create(key, value, note string, organizationID string, projectIDs []string) (*SecretResponse, error) {
	command := Command{
		Secrets: &SecretsCommand{
			Create: &SecretCreateRequest{
				Key:            key,
				Value:          value,
				Note:           note,
				OrganizationID: organizationID,
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

func (s *Secrets) List(organizationID string) (*SecretIdentifiersResponse, error) {
	command := Command{
		Secrets: &SecretsCommand{
			List: &SecretIdentifiersRequest{
				OrganizationID: organizationID,
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

func (s *Secrets) Update(id string, key, value, note string, organizationID string, projectIDs []string) (*SecretResponse, error) {
	oldSecret, err := s.Get(id)
	if err != nil {
		return nil, fmt.Errorf("cannot update secret: failed to fetch current value for version history: %w", err)
	}

	valueChanged := value != oldSecret.Value

	command := Command{
		Secrets: &SecretsCommand{
			Update: &SecretPutRequest{
				ID:             id,
				Key:            key,
				Value:          value,
				Note:           note,
				OrganizationID: organizationID,
				ProjectIDS:     projectIDs,
				ValueChanged:   valueChanged,
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

func (s *Secrets) Sync(organizationID string, lastSyncedDate *time.Time) (*SecretsSyncResponse, error) {
	command := Command{
		Secrets: &SecretsCommand{
			Sync: &SecretsSyncRequest{
				OrganizationID: organizationID,
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
