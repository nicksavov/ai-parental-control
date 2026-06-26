package main

import (
	"context"
	"sync"
)

// Notifier wakes a waiting parent when a new alert arrives for it, so the stream
// endpoint can long-poll instead of busy-looping. The memory implementation
// works for a single node; the Redis implementation (redis.go) fans out across
// nodes via pub/sub.
type Notifier interface {
	Notify(ctx context.Context, recipientID string)
	// Wait blocks until a notification for recipientID arrives or ctx is done.
	Wait(ctx context.Context, recipientID string)
}

type memNotifier struct {
	mu      sync.Mutex
	waiters map[string][]chan struct{}
}

func newMemNotifier() *memNotifier {
	return &memNotifier{waiters: map[string][]chan struct{}{}}
}

func (n *memNotifier) Notify(_ context.Context, recipientID string) {
	n.mu.Lock()
	defer n.mu.Unlock()
	for _, ch := range n.waiters[recipientID] {
		close(ch)
	}
	delete(n.waiters, recipientID)
}

func (n *memNotifier) Wait(ctx context.Context, recipientID string) {
	ch := make(chan struct{})
	n.mu.Lock()
	n.waiters[recipientID] = append(n.waiters[recipientID], ch)
	n.mu.Unlock()
	select {
	case <-ch:
	case <-ctx.Done():
		n.remove(recipientID, ch)
	}
}

func (n *memNotifier) remove(recipientID string, ch chan struct{}) {
	n.mu.Lock()
	defer n.mu.Unlock()
	list := n.waiters[recipientID]
	for i, c := range list {
		if c == ch {
			n.waiters[recipientID] = append(list[:i], list[i+1:]...)
			break
		}
	}
}
