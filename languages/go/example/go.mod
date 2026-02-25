module example

replace github.com/bitwarden/sdk-go/v2 => ../

go 1.21

require (
	github.com/bitwarden/sdk-go/v2 v2.0.0-00010101000000-000000000000
	github.com/gofrs/uuid v4.4.0+incompatible
)
