use super::*;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::RwLock;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_sfu_creation() {
        let packet_processor = MockPacketProcessor::new();
        let sfu = SFU::new(packet_processor);
        
        assert!(sfu.rooms.read().await.is_empty());
        assert!(sfu.peers.read().await.is_empty());
        assert!(sfu.tracks.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_room_creation() {
        let packet_processor = MockPacketProcessor::new();
        let sfu = SFU::new(packet_processor);
        
        let tenant_id = Uuid::new_v4();
        let room_id = sfu.create_room(tenant_id, 10).await.unwrap();
        
        assert!(sfu.rooms.read().await.contains_key(&room_id.0));
    }

    #[tokio::test]
    async fn test_peer_addition() {
        let packet_processor = MockPacketProcessor::new();
        let sfu = SFU::new(packet_processor);
        
        let tenant_id = Uuid::new_v4();
        let room_id = sfu.create_room(tenant_id, 10).await.unwrap();
        
        let peer_id = Uuid::new_v4();
        sfu.add_peer(room_id.0, peer_id).await.unwrap();
        
        let room = sfu.rooms.read().await.get(&room_id.0).unwrap();
        assert!(room.peers.contains_key(&peer_id));
    }

    #[tokio::test]
    async fn test_packet_handling() {
        let packet_processor = MockPacketProcessor::new();
        let sfu = SFU::new(packet_processor);
        
        let packet = RTPPacket {
            ssrc: 1234,
            sequence_number: 1,
            timestamp: 1000,
            payload_type: 96,
            payload: vec![1, 2, 3, 4],
            marker: false,
            extension: None,
        };
        
        let result = sfu.handle_packet(packet, Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_room_stats() {
        let packet_processor = MockPacketProcessor::new();
        let sfu = SFU::new(packet_processor);
        
        let tenant_id = Uuid::new_v4();
        let room_id = sfu.create_room(tenant_id, 10).await.unwrap();
        
        let stats = sfu.get_room_stats(room_id.0).await.unwrap();
        assert_eq!(stats.room_id, room_id.0);
        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.track_count, 0);
    }

    struct MockPacketProcessor;
    impl MockPacketProcessor {
        fn new() -> Self { Self }
    }

    impl PacketProcessor for MockPacketProcessor {
        fn process_rtp(&self, _packet: RTPPacket) -> Result<(), PacketError> {
            Ok(())
        }
        fn process_rtcp(&self, _packet: RTCPPacket) -> Result<(), PacketError> {
            Ok(())
        }
        fn get_forwarding_strategy(&self, _track_id: Uuid) -> ForwardingStrategy {
            ForwardingStrategy::Unicast { peer_ids: vec![] }
        }
    }

    #[tokio::test]
    async fn test_auth_service_creation() {
        let db_pool = PgPool::connect("host=localhost user=postgres").await.unwrap();
        let auth_service = AuthService::new(db_pool);
        
        assert!(auth_service.db_pool.is_valid());
    }

    #[tokio::test]
    async fn test_user_registration() {
        let db_pool = PgPool::connect("host=localhost user=postgres").await.unwrap();
        let auth_service = AuthService::new(db_pool);
        
        let user_id = auth_service.register_user("test", "test@example.com", "password").await.unwrap();
        assert!(user_id != Uuid::new_v4());
    }

    #[tokio::test]
    async fn test_user_authentication() {
        let db_pool = PgPool::connect("host=localhost user=postgres").await.unwrap();
        let auth_service = AuthService::new(db_pool);
        
        auth_service.register_user("test", "test@example.com", "password").await.unwrap();
        
        let user = auth_service.authenticate("test", "password").await.unwrap();
        assert_eq!(user.username, "test");
    }

    #[tokio::test]
    async fn test_token_validation() {
        let db_pool = PgPool::connect("host=localhost user=postgres").await.unwrap();
        let auth_service = AuthService::new(db_pool);
        
        let user = auth_service.register_user("test", "test@example.com", "password").await.unwrap();
        let token = auth_service.create_token(&User {
            id: user,
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![],
            tenant_id: None,
            created_at: chrono::Utc::now(),
        }).await.unwrap();
        
        let validated_user = auth_service.validate_token(&token).await.unwrap();
        assert_eq!(validated_user.username, "test");
    }

    #[tokio::test]
    async fn test_auth_authorization() {
        let db_pool = PgPool::connect("host=localhost user=postgres").await.unwrap();
        let auth_service = AuthService::new(db_pool);
        
        let user = User {
            id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![Role::Viewer],
            tenant_id: None,
            created_at: chrono::Utc::now(),
        };
        
        let authorized = auth_service.authorize(&user, "rooms", "subscribe").await.unwrap();
        assert!(authorized);
    }
}