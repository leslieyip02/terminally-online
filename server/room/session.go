package room

import (
	"fmt"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

type SessionManager struct {
	secret []byte
}

func NewSessionManager(secret []byte) SessionManager {
	return SessionManager{
		secret: secret,
	}
}

func (m *SessionManager) createToken(roomId string) (string, error) {
	claims := jwt.MapClaims{
		"room": roomId,
		"exp":  time.Now().Add(5 * time.Minute).Unix(),
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(m.secret)
}

func (m *SessionManager) validateToken(tokenString string) (string, error) {
	token, err := jwt.Parse(tokenString, func(t *jwt.Token) (any, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method")
		}
		return m.secret, nil
	})

	if err != nil || !token.Valid {
		return "", fmt.Errorf("invalid token")
	}

	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok {
		return "", fmt.Errorf("invalid claims")
	}

	roomId, ok := claims["room"].(string)
	if !ok {
		return "", fmt.Errorf("room claim missing or invalid")
	}

	return roomId, nil
}
