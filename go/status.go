package gwp

// GQLSTATUS constants (ISO/IEC 39075 Chapter 23).
const (
	Success            = "00000"
	OmittedResult      = "00001"
	Warning            = "01000"
	NoData             = "02000"
	InvalidSyntax      = "42001"
	GraphTypeViolation = "G2000"
)

// StatusClass extracts the 2-character class from a 5-character GQLSTATUS code.
func StatusClass(code string) string {
	if len(code) < 2 {
		return code
	}
	return code[:2]
}

// IsSuccess checks if the status indicates success (class 00).
func IsSuccess(code string) bool {
	return StatusClass(code) == "00"
}

// IsWarning checks if the status indicates a warning (class 01).
func IsWarning(code string) bool {
	return StatusClass(code) == "01"
}

// IsNoData checks if the status indicates no data (class 02).
func IsNoData(code string) bool {
	return StatusClass(code) == "02"
}

// IsException checks if the status indicates an exception.
func IsException(code string) bool {
	cls := StatusClass(code)
	return cls != "00" && cls != "01" && cls != "02"
}
