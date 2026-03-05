#!/bin/bash

# DRMP - Distributed Realtime Media Platform
# Build script for all services

set -e

echo "🚀 Building DRMP - Distributed Realtime Media Platform"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    print_status "Docker is running"
}

# Check if kubectl is available
check_kubectl() {
    if ! command -v kubectl &> /dev/null; then
        print_warning "kubectl not found. Skipping Kubernetes validation."
        return
    fi
    
    if kubectl cluster-info > /dev/null 2>&1; then
        print_status "kubectl is connected to a cluster"
    else
        print_warning "kubectl is not connected to a cluster"
    fi
}

# Build all services
build_all() {
    echo "📦 Building all services..."
    
    # Build shared library first
    if [ -d "shared" ]; then
        print_status "Building shared library..."
        (cd shared && cargo build --release)
    fi
    
    # Build each service
    services=(media-edge sfu control-plane recording auth gateway)
    
    for service in "${services[@]}"; do
        if [ -d "services/$service" ]; then
            print_status "Building $service..."
            (cd "services/$service" && cargo build --release)
        fi
    done
    
    print_status "All services built successfully!"
}

# Build Docker images
build_docker_images() {
    echo "🌳 Building Docker images..."
    
    # Build base image for Rust services
    print_status "Building base Rust image..."
    docker build -t drmp/rust-base:latest -f docker/rust-base.Dockerfile .
    
    # Build each service image
    services=(media-edge sfu control-plane recording auth gateway)
    
    for service in "${services[@]}"; do
        if [ -d "services/$service" ]; then
            print_status "Building $service image..."
            docker build -t drmp/$service:latest -f "services/$service/Dockerfile" .
        fi
    done
    
    print_status "All Docker images built successfully!"
}

# Run tests
run_tests() {
    echo "🧪 Running tests..."
    
    # Run tests for shared library
    if [ -d "shared" ]; then
        print_status "Running shared library tests..."
        (cd shared && cargo test --release)
    fi
    
    # Run tests for each service
    services=(media-edge sfu control-plane recording auth gateway)
    
    for service in "${services[@]}"; do
        if [ -d "services/$service" ]; then
            print_status "Running $service tests..."
            (cd "services/$service" && cargo test --release)
        fi
    done
    
    print_status "All tests passed!"
}

# Start local development environment
start_local() {
    echo "🚀 Starting local development environment..."
    
    # Check if docker-compose is available
    if ! command -v docker-compose &> /dev/null; then
        print_error "docker-compose not found. Please install docker-compose and try again."
        exit 1
    fi
    
    # Start services
    print_status "Starting services with docker-compose..."
    docker-compose -f docker/docker-compose.dev.yml up --build
}

# Start production environment
start_production() {
    echo "🏭 Starting production environment..."
    
    # Check if kubectl is available
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl not found. Please install kubectl and try again."
        exit 1
    fi
    
    # Apply Kubernetes manifests
    print_status "Applying Kubernetes manifests..."
    kubectl apply -f k8s/
    
    print_status "Production environment started!"
}

# Stop local development environment
stop_local() {
    echo "🛑 Stopping local development environment..."
    
    # Stop services
    print_status "Stopping services..."
    docker-compose -f docker/docker-compose.dev.yml down
    
    print_status "Local environment stopped!"
}

# Stop production environment
stop_production() {
    echo "🛑 Stopping production environment..."
    
    # Check if kubectl is available
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl not found. Please install kubectl and try again."
        exit 1
    fi
    
    # Delete Kubernetes resources
    print_status "Deleting Kubernetes resources..."
    kubectl delete -f k8s/ 2>/dev/null || true
    
    print_status "Production environment stopped!"
}

# Show help
show_help() {
    echo "DRMP - Distributed Realtime Media Platform"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build       Build all services and Docker images"
    echo "  test        Run all tests"
    echo "  start       Start local development environment"
    echo "  stop        Stop local development environment"
    echo "  deploy      Deploy to Kubernetes production"
    echo "  undeploy    Undeploy from Kubernetes production"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build     # Build everything"
    echo "  $0 start     # Start local development"
    echo "  $0 deploy    # Deploy to production"
}

# Main script logic
case "${1:-help}" in
    build)
        check_docker
        build_all
        build_docker_images
        ;;
    test)
        check_docker
        run_tests
        ;;
    start)
        check_docker
        start_local
        ;;
    stop)
        stop_local
        ;;
    deploy)
        check_docker
        check_kubectl
        start_production
        ;;
    undeploy)
        stop_production
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac