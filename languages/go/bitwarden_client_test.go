package sdk

import (
	"os"
	"testing"
	"time"

	"github.com/gofrs/uuid"
)

func TestBitwardenClient(t *testing.T) {
	apiURL := os.Getenv("API_URL")
	identityURL := os.Getenv("IDENTITY_URL")
	accessToken := os.Getenv("ACCESS_TOKEN")
	organizationIDStr := os.Getenv("ORGANIZATION_ID")
	stateFile := os.Getenv("STATE_FILE")

	bitwardenClient, err := NewBitwardenClient(&apiURL, &identityURL)
	if err != nil {
		t.Errorf("Failed to create Bitwarden client: %v", err)
	}
	defer bitwardenClient.Close()

	err = bitwardenClient.AccessTokenLogin(accessToken, &stateFile)
	if err != nil {
		t.Errorf("AccessTokenLogin failed: %v", err)
	}

	organizationID, err := uuid.FromString(organizationIDStr)
	if err != nil {
		t.Errorf("Failed to parse organization ID: %v", err)
	}

	// --- generator ---
	request := PasswordGeneratorRequest{
		AvoidAmbiguous: true,
		Length:         32,
		Lowercase:      true,
		MinLowercase:   ptr(int64(2)),
		MinNumber:      ptr(int64(2)),
		MinSpecial:     ptr(int64(2)),
		MinUppercase:   ptr(int64(2)),
		Numbers:        true,
		Special:        true,
		Uppercase:      true,
	}

	password, err := bitwardenClient.Generators().GeneratePassword(request)
	if err != nil || len(*password) != 32 {
		t.Errorf("generate failed: %v", err)
	}

	// --- secrets ---
	// list; should return a list of secret IDs (without the values)
	secretList, err := bitwardenClient.Secrets().List(organizationID.String())
	if err != nil || len(secretList.Data) == 0 {
		t.Errorf("secret list failed: %v", err)
		t.Errorf("secret list data: %v", secretList.Data)
	}

	// get; should return a secret whose key is "btw" (from the fake-server)
	secret, err := bitwardenClient.Secrets().Get(secretList.Data[0].ID)
	hardCodedSecretKey := "btw" // embedded in the fake-server
	if err != nil || secret.Key != hardCodedSecretKey {
		t.Errorf("secret get failed: %v", err)
		t.Errorf("secret key: %s", secret.Key)
	}

	// getByIds; should return secret data for the given IDs
	secretIDs := []string{uuid.Must(uuid.NewV4()).String(), uuid.Must(uuid.NewV4()).String()}
	hardCodedSecretKey = "FERRIS" // embedded in the fake-server
	secrets, err := bitwardenClient.Secrets().GetByIDS(secretIDs)
	if err != nil || secrets.Data[0].Key != hardCodedSecretKey {
		t.Errorf("secret getByIds failed: %v", err)
		t.Errorf("secret key: %s", secrets.Data[0].Key)
	}

	// create; should return a secret with the given key, value, and note
	newProjectID, _ := uuid.NewV4() // random project ID is fine; the fake-server doesn't validate it

	secret, err = bitwardenClient.Secrets().Create("testKey", "testValue", "testNote", organizationID.String(), []string{newProjectID.String()})
	if err != nil || secret.Key != "testKey" || secret.Value != "testValue" || secret.Note != "testNote" {
		t.Errorf("secret create failed: %v", err)
	}

	// update; should return a secret with the updated key, value, and note
	updatedSecret, err := bitwardenClient.Secrets().Update(secret.ID, "updatedKey", "updatedValue", "updatedNote", organizationID.String(), []string{})
	if err != nil || updatedSecret.Key != "updatedKey" || updatedSecret.Value != "updatedValue" || updatedSecret.Note != "updatedNote" || updatedSecret.ProjectID != nil {
		t.Errorf("secret update failed: %v", err)
	}

	// delete; should delete the secret and return an empty response
	res, err := bitwardenClient.Secrets().Delete([]string{
		uuid.Must(uuid.NewV4()).String(),
	})
	if err != nil {
		t.Errorf("secret delete failed: %v", err)
		t.Errorf("expected nil response, got: %v", res)
	}

	// sync; should return new/modified secrets from a given point in time
	syncedSecrets, err := bitwardenClient.Secrets().Sync(organizationID.String(), nil)
	if err != nil || syncedSecrets.HasChanges == false {
		t.Errorf("secret initial sync failed: %v", err)
		t.Errorf("secret hasChanges: %v", syncedSecrets.HasChanges)
	}

	lastSyncTime := time.Now()
	newSyncedSecrets, err := bitwardenClient.Secrets().Sync(organizationID.String(), &lastSyncTime)
	if err != nil || newSyncedSecrets.HasChanges == true {
		t.Errorf("secret sync with lastSyncTime failed: %v", err)
		t.Errorf("secret hasChanges: %v", newSyncedSecrets.HasChanges)
	}

	// --- projects ---
	// list; should return a list of project IDs
	projectList, err := bitwardenClient.Projects().List(organizationID.String())
	if err != nil || len(projectList.Data) == 0 {
		t.Errorf("project list failed: %v", err)
		t.Errorf("project list data: %v", projectList.Data)
	}

	// get; should return a project with the given ID
	_, err = bitwardenClient.Projects().Get(projectList.Data[0].ID)
	if err != nil {
		t.Errorf("project get failed: %v", err)
	}

	// create; should return a project with the given name
	project, err := bitwardenClient.Projects().Create(organizationID.String(), "testProject")
	if err != nil || project.Name != "testProject" {
		t.Errorf("project create failed: %v", err)
		t.Errorf("expected project name: testProject, got: %s", project.Name)
	}

	// update; should return a project with the updated name
	project, err = bitwardenClient.Projects().Update(project.ID, organizationID.String(), "updatedProject")
	if err != nil || project.Name != "updatedProject" {
		t.Errorf("project update failed: %v", err)
		t.Errorf("expected project name: updatedProject, got: %s", project.Name)
	}

	// delete; should delete the project
	projectRes, err := bitwardenClient.Projects().Delete([]string{project.ID})
	if err != nil {
		t.Errorf("project delete failed: %v", err)
		t.Errorf("expected deleted project ID: %s, got: %v", project.ID, projectRes.Data)
	}
}

// Helper functions
func ptr(i int64) *int64 {
	return &i
}
