module example

replace github.com/bitwarden/sdk-go => ../

go 1.21

require (
	github.com/bitwarden/sdk-go v0.1.1
	github.com/gofrs/uuid v4.4.0+incompatible
)
