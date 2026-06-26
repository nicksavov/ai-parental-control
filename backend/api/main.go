// Command api is the thin coordination backend for ai-parental-control.
//
// It does pairing, auth, policy distribution, and encrypted-alert relay. It
// never decrypts an alert: alert envelopes are stored and forwarded as opaque
// bytes, keyed by recipient. See packages/proto/openapi.yaml for the surface.
package main

import (
	"encoding/json"
	"log"
	"net/http"
	"os"
)

type server struct {
	secret []byte
	store  *store
}

func newServer(secret []byte, st *store) *server {
	return &server{secret: secret, store: st}
}

func (s *server) routes() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("POST /v1/auth/token", s.handleToken)
	mux.HandleFunc("POST /v1/family/children", s.requireParent(s.handleCreateChild))
	mux.HandleFunc("POST /v1/pairing/codes", s.requireParent(s.handleCreateCode))
	mux.HandleFunc("POST /v1/pairing/claim", s.handleClaim)
	mux.HandleFunc("PUT /v1/devices/{id}/policy", s.requireParent(s.handlePutPolicy))
	mux.HandleFunc("GET /v1/devices/{id}/policy", s.requireDevice(s.handleGetPolicy))
	mux.HandleFunc("POST /v1/alerts", s.requireDevice(s.handleSubmitAlert))
	mux.HandleFunc("GET /v1/alerts/stream", s.requireParent(s.handleStreamAlerts))
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
	return mux
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

func writeError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, map[string]string{"error": msg})
}

func main() {
	secret := []byte(os.Getenv("JWT_SIGNING_KEY"))
	if len(secret) == 0 {
		log.Fatal("JWT_SIGNING_KEY must be set")
	}
	addr := ":8080"
	if p := os.Getenv("API_PORT"); p != "" {
		addr = ":" + p
	}
	srv := newServer(secret, newStore())
	log.Printf("api listening on %s", addr)
	log.Fatal(http.ListenAndServe(addr, srv.routes()))
}
