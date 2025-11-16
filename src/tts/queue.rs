/// TTS request queue management

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use crate::tts::{TTSRequest, QueueStatus};

const MAX_QUEUE_SIZE: usize = 50;

/// Thread-safe TTS queue
pub struct TTSQueue {
    queue: Arc<RwLock<VecDeque<TTSRequest>>>,
    current: Arc<RwLock<Option<TTSRequest>>>,
    playing: Arc<RwLock<bool>>,
}

impl TTSQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            current: Arc::new(RwLock::new(None)),
            playing: Arc::new(RwLock::new(false)),
        }
    }

    /// Add a request to the queue
    /// Returns Err if queue is full
    pub fn enqueue(&self, request: TTSRequest) -> Result<(), String> {
        let mut queue = self.queue.write().unwrap();

        if queue.len() >= MAX_QUEUE_SIZE {
            return Err(format!("Queue is full (max {})", MAX_QUEUE_SIZE));
        }

        queue.push_back(request);
        log::debug!("Request added to queue. Queue length: {}", queue.len());

        Ok(())
    }

    /// Get the next request from the queue
    pub fn dequeue(&self) -> Option<TTSRequest> {
        let mut queue = self.queue.write().unwrap();
        let request = queue.pop_front();

        if let Some(ref req) = request {
            log::debug!("Request dequeued. Remaining: {}", queue.len());
            *self.current.write().unwrap() = Some(req.clone());
        }

        request
    }

    /// Clear all queued requests
    pub fn clear(&self) {
        let mut queue = self.queue.write().unwrap();
        let count = queue.len();
        queue.clear();
        log::info!("Queue cleared. Removed {} requests", count);
    }

    /// Get the current queue status
    pub fn status(&self) -> QueueStatus {
        let current = self.current.read().unwrap().clone();
        let queue_length = self.queue.read().unwrap().len();
        let playing = *self.playing.read().unwrap();

        QueueStatus {
            current,
            queue_length,
            playing,
        }
    }

    /// Set playing state
    pub fn set_playing(&self, playing: bool) {
        *self.playing.write().unwrap() = playing;
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.read().unwrap().is_empty()
    }

    /// Get queue length
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.queue.read().unwrap().len()
    }

    /// Mark current request as complete
    pub fn complete_current(&self) {
        *self.current.write().unwrap() = None;
        self.set_playing(false);
    }
}

impl Default for TTSQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request(text: &str) -> TTSRequest {
        TTSRequest::new(text.to_string(), "test-voice".to_string(), 1.0)
    }

    #[test]
    fn test_queue_creation() {
        let queue = TTSQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_enqueue_dequeue() {
        let queue = TTSQueue::new();
        let req1 = create_test_request("Hello");
        let req2 = create_test_request("World");

        queue.enqueue(req1.clone()).unwrap();
        queue.enqueue(req2.clone()).unwrap();

        assert_eq!(queue.len(), 2);

        let dequeued1 = queue.dequeue().unwrap();
        assert_eq!(dequeued1.text, "Hello");

        let dequeued2 = queue.dequeue().unwrap();
        assert_eq!(dequeued2.text, "World");

        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_limit() {
        let queue = TTSQueue::new();

        // Fill queue to limit
        for i in 0..MAX_QUEUE_SIZE {
            let req = create_test_request(&format!("Message {}", i));
            queue.enqueue(req).unwrap();
        }

        // Next enqueue should fail
        let overflow_req = create_test_request("Overflow");
        assert!(queue.enqueue(overflow_req).is_err());
    }

    #[test]
    fn test_clear_queue() {
        let queue = TTSQueue::new();

        for i in 0..5 {
            let req = create_test_request(&format!("Message {}", i));
            queue.enqueue(req).unwrap();
        }

        assert_eq!(queue.len(), 5);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_status() {
        let queue = TTSQueue::new();
        let req = create_test_request("Test");

        queue.enqueue(req).unwrap();
        let status = queue.status();

        assert_eq!(status.queue_length, 1);
        assert!(!status.playing);
    }
}
