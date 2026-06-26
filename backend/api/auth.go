package main

import (
	"net/http"
	"strings"
)

func bearer(r *http.Request) string {
	if after, found := strings.CutPrefix(r.Header.Get("Authorization"), "Bearer "); found {
		return after
	}
	return ""
}

type parentHandler func(w http.ResponseWriter, r *http.Request, parentID string)

func (s *server) requireParent(h parentHandler) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		c, err := s.parse(bearer(r))
		if err != nil || c.Role != "parent" || c.Typ != "access" {
			writeError(w, http.StatusUnauthorized, "unauthorized")
			return
		}
		h(w, r, c.Sub)
	}
}

type deviceHandler func(w http.ResponseWriter, r *http.Request, childDeviceID string)

func (s *server) requireDevice(h deviceHandler) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		c, err := s.parse(bearer(r))
		if err != nil || c.Role != "device" || c.Typ != "access" {
			writeError(w, http.StatusUnauthorized, "unauthorized")
			return
		}
		h(w, r, c.Sub)
	}
}
