package gwp

import "testing"

func TestIsSuccess(t *testing.T) {
	if !IsSuccess(Success) {
		t.Fatal("expected success")
	}
	if IsException(Success) {
		t.Fatal("expected not exception")
	}
}

func TestOmitted(t *testing.T) {
	if !IsSuccess(OmittedResult) {
		t.Fatal("expected success for omitted")
	}
}

func TestWarning(t *testing.T) {
	if !IsWarning(Warning) {
		t.Fatal("expected warning")
	}
	if IsSuccess(Warning) {
		t.Fatal("expected not success")
	}
	if IsException(Warning) {
		t.Fatal("expected not exception")
	}
}

func TestNoData(t *testing.T) {
	if !IsNoData(NoData) {
		t.Fatal("expected no data")
	}
	if IsSuccess(NoData) {
		t.Fatal("expected not success")
	}
}

func TestException(t *testing.T) {
	if !IsException(InvalidSyntax) {
		t.Fatal("expected exception")
	}
	if IsSuccess(InvalidSyntax) {
		t.Fatal("expected not success")
	}
}

func TestGraphTypeViolation(t *testing.T) {
	if !IsException(GraphTypeViolation) {
		t.Fatal("expected exception")
	}
}

func TestStatusClass(t *testing.T) {
	tests := []struct {
		code string
		want string
	}{
		{"00000", "00"},
		{"42001", "42"},
		{"G2000", "G2"},
	}
	for _, tt := range tests {
		got := StatusClass(tt.code)
		if got != tt.want {
			t.Fatalf("StatusClass(%q) = %q, want %q", tt.code, got, tt.want)
		}
	}
}
