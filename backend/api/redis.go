package main

import (
	"context"

	"github.com/redis/go-redis/v9"
)

// redisNotifier fans out alert-arrival notifications across backend nodes using
// Redis pub/sub. Selected when REDIS_URL is set. The payload is empty: it only
// signals "drain your inbox", never alert content.
type redisNotifier struct {
	client *redis.Client
}

func newRedisNotifier(url string) (*redisNotifier, error) {
	opt, err := redis.ParseURL(url)
	if err != nil {
		return nil, err
	}
	return &redisNotifier{client: redis.NewClient(opt)}, nil
}

func channelFor(recipientID string) string { return "alerts:" + recipientID }

func (n *redisNotifier) Notify(ctx context.Context, recipientID string) {
	_ = n.client.Publish(ctx, channelFor(recipientID), "").Err()
}

func (n *redisNotifier) Wait(ctx context.Context, recipientID string) {
	sub := n.client.Subscribe(ctx, channelFor(recipientID))
	defer sub.Close()
	select {
	case <-sub.Channel():
	case <-ctx.Done():
	}
}
