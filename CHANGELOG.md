# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Enhanced
- **Production Config Compatibility**: Comprehensive enhancements to production config support
- **Secrets Management**: Enhanced automatic injection of secrets as environment variables during container startup
- **LiteFS Production Adaptation**: Improved production `litefs.yml` config adaptation with robust validation and error handling
- **Service Discovery**: Automatic DNS registration/unregistration for .internal domain resolution
- **Docker Integration**: Seamless secrets loading integration with container creation process

### Improved
- **LiteFS Config Validation**: Added validation for proxy targets, lease types, and configuration paths
- **Error Recovery**: Enhanced fallback mechanisms when production config adaptation fails
- **Development Experience**: Better logging and debug integration for troubleshooting production config issues

## [0.1.2] - 2024-06-23

### Fixed
- **Docker Container Creation**: Fixed "Failed to create container" errors by implementing proper volume mounting with absolute paths
- **Port Detection**: Deployment now shows actual Docker-assigned ports in clickable URLs instead of static fly.toml ports
- **Machine Reuse**: Added logic to detect and reuse existing machines instead of always creating new ones
- **Volume Database Creation**: Automatically creates SQLite database files for `/litefs` volumes during deployment
- **UUID Naming**: Implemented UUID-based machine naming to prevent naming conflicts

### Changed
- **Volume Path Structure**: Updated volume mapping to use absolute paths and maintain existing minifly-data structure
- **Port Allocation**: Enhanced automatic port allocation (port 0) to work reliably with container reuse
- **Container Lifecycle**: Improved container lifecycle management with proper cleanup and reuse logic

### Added
- **Port Detection Functions**: Added multiple methods to detect actual Docker-assigned ports
- **Container Inspection**: Enhanced container inspection capabilities for better port detection
- **Database Auto-creation**: Automatically creates database files when mounting volumes for database applications

## [0.1.1] - 2024-06-22

### Added
- Enhanced `init` command with 5 project templates
- Real-time log streaming with Server-Sent Events
- Graceful shutdown handling
- Complete CLI reference documentation
- API reference documentation
- Documentation site at https://minifly-docs.fly.dev

### Changed
- Updated repository URL to https://github.com/NoHeadDotDev/minifly
- Improved error messages and logging

### Fixed
- Type mismatches in serve command
- Raw string literal syntax in init templates

## [0.1.0] - 2024-06-22

### Added
- Initial release
- Complete Fly.io Machines API compatibility
- LiteFS integration for distributed SQLite
- Docker-based container management
- Multi-region simulation
- Comprehensive CLI with all major commands
- Structured logging with correlation IDs