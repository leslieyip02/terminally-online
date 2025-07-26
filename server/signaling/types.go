package signaling

type CandidateSink interface {
	OnCandidateMessage(message *SignalMessage)
}