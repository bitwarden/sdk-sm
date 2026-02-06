package sdk

type GeneratorsInterface interface {
	GeneratePassword(request PasswordGeneratorRequest) (*string, error)
}

type Generators struct {
	CommandRunner CommandRunnerInterface
}

func NewGenerators(commandRunner CommandRunnerInterface) *Generators {
	return &Generators{CommandRunner: commandRunner}
}

func (s *Generators) executeCommand(command Command, target any) error {
	responseStr, err := s.CommandRunner.RunCommand(command)
	if err != nil {
		return err
	}
	return checkSuccessAndError(responseStr, target)
}

func (s *Generators) GeneratePassword(request PasswordGeneratorRequest) (*string, error) {
	command := Command{
		Generators: &GeneratorsCommand{
			GeneratePassword: request,
		},
	}

	var response string
	if err := s.executeCommand(command, &response); err != nil {
		return nil, err
	}
	return &response, nil
}
