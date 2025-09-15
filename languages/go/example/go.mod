module example

replace github.com/bitwarden/sdk-go => ../

go 1.21

require (
	github.com/bitwarden/sdk-go v1.0.2
	github.com/gofrs/uuid v4.4.0+incompatible
)
