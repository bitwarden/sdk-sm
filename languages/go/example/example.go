package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"

	sdk "github.com/bitwarden/sdk-go/v2"
	"github.com/gofrs/uuid"
)

func main() {
	// Configuring the URLS is optional, set them to nil to use the default values
	apiURL := os.Getenv("API_URL")
	identityURL := os.Getenv("IDENTITY_URL")

	bitwardenClient, _ := sdk.NewBitwardenClient(&apiURL, &identityURL)

	accessToken := os.Getenv("ACCESS_TOKEN")
	organizationIDStr := os.Getenv("ORGANIZATION_ID")
	projectName := os.Getenv("PROJECT_NAME")

	// Configuring the stateFile is optional, pass nil
	// in AccessTokenLogin() to not use state
	stateFile := os.Getenv("STATE_FILE")

	if projectName == "" {
		projectName = "NewTestProject" // default value
	}

	err := bitwardenClient.AccessTokenLogin(accessToken, &stateFile)
	if err != nil {
		panic(err)
	}

	organizationID, err := uuid.FromString(organizationIDStr)
	if err != nil {
		panic(err)
	}

	project, err := bitwardenClient.Projects().Create(organizationID.String(), projectName)
	if err != nil {
		panic(err)
	}
	fmt.Println(project)
	projectID := project.ID
	fmt.Println(projectID)

	if _, err = bitwardenClient.Projects().List(organizationID.String()); err != nil {
		panic(err)
	}

	if _, err = bitwardenClient.Projects().Get(projectID); err != nil {
		panic(err)
	}

	if _, err = bitwardenClient.Projects().Update(projectID, organizationID.String(), projectName+"2"); err != nil {
		panic(err)
	}

	key := "key"
	value := "value"
	note := "note"

	secret, err := bitwardenClient.Secrets().Create(key, value, note, organizationID.String(), []string{projectID})
	if err != nil {
		panic(err)
	}
	secretID := secret.ID

	if _, err = bitwardenClient.Secrets().List(organizationID.String()); err != nil {
		panic(err)
	}

	if _, err = bitwardenClient.Secrets().Get(secretID); err != nil {
		panic(err)
	}

	if _, err = bitwardenClient.Secrets().Update(secretID, key, value, note, organizationID.String(), []string{projectID}); err != nil {
		panic(err)
	}

	if _, err = bitwardenClient.Secrets().Delete([]string{secretID}); err != nil {
		panic(err)
	}

	if _, err = bitwardenClient.Projects().Delete([]string{projectID}); err != nil {
		panic(err)
	}

	secretIdentifiers, err := bitwardenClient.Secrets().List(organizationID.String())
	if err != nil {
		panic(err)
	}

	// Get secrets with a list of IDs
	secretIDs := make([]string, len(secretIdentifiers.Data))
	for i, identifier := range secretIdentifiers.Data {
		secretIDs[i] = identifier.ID
	}

	secrets, err := bitwardenClient.Secrets().GetByIDS(secretIDs)
	if err != nil {
		log.Fatalf("Error getting secrets: %v", err)
	}

	jsonSecrets, err := json.MarshalIndent(secrets, "", "  ")
	if err != nil {
		log.Fatalf("Error marshalling secrets to JSON: %v", err)
	}

	fmt.Println(string(jsonSecrets))

	// Generate a password which can be used as a secret value
	request := sdk.PasswordGeneratorRequest{
		AvoidAmbiguous: true,
		Length:         64,
		Lowercase:      true,
		MinLowercase:   new(int64),
		MinNumber:      new(int64),
		MinSpecial:     new(int64),
		MinUppercase:   new(int64),
		Numbers:        true,
		Special:        true,
		Uppercase:      true,
	}
	password, err := bitwardenClient.Generators().GeneratePassword(request)

	if err != nil {
		panic(err)
	}

	fmt.Println(*password)

	defer bitwardenClient.Close()
}
