package sdk

import "fmt"

type ProjectsInterface interface {
	Create(name string) (*ProjectResponse, error)
	List() (*ProjectsResponse, error)
	Get(projectID string) (*ProjectResponse, error)
	Update(projectID string, name string) (*ProjectResponse, error)
	Delete(projectIDs []string) (*ProjectsDeleteResponse, error)
}

type Projects struct {
	CommandRunner CommandRunnerInterface
	Client        BitwardenClientInterface
}

func NewProjects(commandRunner CommandRunnerInterface, client BitwardenClientInterface) *Projects {
	return &Projects{CommandRunner: commandRunner, Client: client}
}

func (p *Projects) Get(id string) (*ProjectResponse, error) {
	command := Command{
		Projects: &ProjectsCommand{
			Get: &ProjectGetRequest{
				ID: id,
			},
		},
	}
	var response ProjectResponse
	if err := p.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (p *Projects) Create(name string) (*ProjectResponse, error) {
	orgID, err := p.Client.GetAccessTokenOrganization()
	if err != nil {
		return nil, err
	}
	if orgID == "" {
		return nil, fmt.Errorf("no organization found in access token")
	}

	command := Command{
		Projects: &ProjectsCommand{
			Create: &ProjectCreateRequest{
				OrganizationID: orgID,
				Name:           name,
			},
		},
	}

	var response ProjectResponse
	if err := p.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (p *Projects) List() (*ProjectsResponse, error) {
	orgID, err := p.Client.GetAccessTokenOrganization()
	if err != nil {
		return nil, err
	}
	if orgID == "" {
		return nil, fmt.Errorf("no organization found in access token")
	}

	command := Command{
		Projects: &ProjectsCommand{
			List: &ProjectsListRequest{
				OrganizationID: orgID,
			},
		},
	}

	var response ProjectsResponse
	if err := p.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (p *Projects) Update(projectID, name string) (*ProjectResponse, error) {
	orgID, err := p.Client.GetAccessTokenOrganization()
	if err != nil {
		return nil, err
	}
	if orgID == "" {
		return nil, fmt.Errorf("no organization found in access token")
	}

	command := Command{
		Projects: &ProjectsCommand{
			Update: &ProjectPutRequest{
				ID:             projectID,
				OrganizationID: orgID,
				Name:           name,
			},
		},
	}

	var response ProjectResponse
	if err := p.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (p *Projects) Delete(projectIDs []string) (*ProjectsDeleteResponse, error) {
	command := Command{
		Projects: &ProjectsCommand{
			Delete: &ProjectsDeleteRequest{
				IDS: projectIDs,
			},
		},
	}

	var response ProjectsDeleteResponse
	if err := p.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}

func (p *Projects) executeCommand(command Command, target any) error {
	responseStr, err := p.CommandRunner.RunCommand(command)
	if err != nil {
		return err
	}
	return checkSuccessAndError(responseStr, target)
}
