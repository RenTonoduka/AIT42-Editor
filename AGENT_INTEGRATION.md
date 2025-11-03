# AIT42 Editor - Agent Integration Guide

**Version**: 1.0.0
**Last Updated**: 2025-01-06
**Target Audience**: Users, Developers leveraging AI agents

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Quick Start](#2-quick-start)
3. [Coordinator: Auto-Select Agents](#3-coordinator-auto-select-agents)
4. [All 49 Agents](#4-all-49-agents)
5. [Usage Patterns](#5-usage-patterns)
6. [Parallel Execution with Tmux](#6-parallel-execution-with-tmux)
7. [Best Practices](#7-best-practices)
8. [Troubleshooting](#8-troubleshooting)

---

## 1. Introduction

### What are AI Agents?

AIT42 Editor integrates **49 specialized AI agents** that can perform development tasks:
- Code generation and review
- Testing and security analysis
- Infrastructure and deployment
- Documentation and planning

### Two Ways to Use Agents

1. **Coordinator (Recommended)**: Just describe your task, Coordinator auto-selects agents
2. **Manual Selection**: Choose specific agents from the palette (`Ctrl+P`)

### Prerequisites

```bash
# Set AIT42 root path
export AIT42_ROOT=/path/to/AIT42
echo 'export AIT42_ROOT=/path/to/AIT42' >> ~/.zshrc

# Verify agent files exist
ls $AIT42_ROOT/.claude/agents/
# Should show 49 .md files
```

---

## 2. Quick Start

### Using Coordinator (Auto-Select)

**No need to know agent names!** Just describe what you want:

```
# Press Ctrl+P in editor
"Implement user authentication API"

→ Coordinator selects: backend-developer
→ Agent executes task
→ Result: Generated files with authentication code
```

### Using Specific Agent

```
# Press Ctrl+P in editor
Type: "backend-developer"
Task: "Implement REST API for user management"

→ Agent executes
→ Result: Generated API files
```

### Example Session

```
1. Open file: ait42-editor src/main.rs
2. Press Ctrl+P
3. Type: "Review this code for security issues"
4. Coordinator selects: security-tester + code-reviewer
5. View results in status bar
```

---

## 3. Coordinator: Auto-Select Agents

### What is Coordinator?

**Coordinator** is a meta-agent that:
- Analyzes your request
- Selects 1-3 optimal agents
- Decides parallel vs. sequential execution
- Executes agents automatically

### How It Works

```
User Request
    ↓
Coordinator Analysis
    ↓
Agent Selection (1-3 agents)
    ↓
Execution Mode (parallel/sequential)
    ↓
Auto-execution
    ↓
Results
```

### Selection Examples

#### Example 1: Simple Request

**Request**: "Implement user authentication API"

**Analysis**:
- Task Type: Implementation
- Tech Stack: Backend, API
- Complexity: Medium

**Selection**: `backend-developer`

**Execution**: Direct (no Tmux)

---

#### Example 2: Complex Request

**Request**: "Design and implement an e-commerce system"

**Analysis**:
- Task Type: Planning + Implementation
- Tech Stack: Full Stack
- Complexity: Complex

**Selection** (Sequential):
1. `system-architect` (design architecture)
2. `api-designer` + `database-designer` (parallel design)
3. `backend-developer` + `frontend-developer` (parallel implementation)
4. `code-reviewer` + `test-generator` (parallel QA)

**Execution**: Tmux (parallel agents)

---

#### Example 3: QA Request

**Request**: "Review and test this code"

**Analysis**:
- Task Type: QA
- Tech Stack: Code Review + Testing
- Complexity: Medium

**Selection** (Parallel):
1. `code-reviewer`
2. `test-generator`
3. `security-tester`

**Execution**: Tmux (3 agents parallel)

---

### Coordinator Keywords

| Keywords | Selected Agent(s) |
|----------|------------------|
| "design", "architecture" | system-architect, api-designer, database-designer |
| "implement", "code" | backend-developer, frontend-developer |
| "API" | api-developer, backend-developer |
| "test", "testing" | test-generator, integration-tester |
| "review", "quality" | code-reviewer |
| "security", "vulnerability" | security-tester, security-scanner |
| "deploy", "CI/CD" | cicd-manager, devops-engineer |
| "refactor", "improve" | refactor-specialist |
| "document", "docs" | tech-writer, doc-reviewer |
| "performance", "optimize" | performance-tester, process-optimizer |

---

## 4. All 49 Agents

### 4.1 Planning & Design (8 Agents)

#### system-architect
**Description**: System design, architecture patterns, technology selection

**Use Cases**:
- Design microservices architecture
- Select tech stack for new project
- Create architecture decision records (ADR)

**Example**:
```
Request: "Design architecture for a real-time chat application"

Output:
- High-level architecture diagram (ASCII)
- Technology recommendations (WebSocket, Redis, PostgreSQL)
- Scalability considerations
- Security architecture
```

---

#### api-designer
**Description**: API design, OpenAPI/Swagger specs, REST/GraphQL

**Use Cases**:
- Design RESTful API endpoints
- Create OpenAPI specification
- Design GraphQL schema

**Example**:
```
Request: "Design API for user management (CRUD)"

Output:
openapi: 3.0.0
paths:
  /users:
    get:
      summary: List users
      parameters:
        - name: page
          in: query
          schema: {type: integer}
      responses:
        200:
          description: User list
          content:
            application/json:
              schema:
                type: array
                items: {$ref: '#/components/schemas/User'}
```

---

#### database-designer
**Description**: Database design, ERD, normalization, indexing

**Use Cases**:
- Design database schema
- Create entity-relationship diagrams
- Optimize indexes

**Example**:
```
Request: "Design database schema for e-commerce (users, products, orders)"

Output:
Tables:
  users:
    - id (PK, UUID)
    - email (UNIQUE, INDEX)
    - password_hash
    - created_at
    - updated_at

  products:
    - id (PK, UUID)
    - name
    - price (DECIMAL)
    - stock (INTEGER)
    - category_id (FK → categories.id, INDEX)

  orders:
    - id (PK, UUID)
    - user_id (FK → users.id, INDEX)
    - total_amount (DECIMAL)
    - status (ENUM: pending, paid, shipped, completed)
    - created_at (INDEX)

  order_items:
    - id (PK, UUID)
    - order_id (FK → orders.id, INDEX)
    - product_id (FK → products.id)
    - quantity (INTEGER)
    - price (DECIMAL)

Indexes:
  - users(email) for login lookups
  - orders(user_id, created_at) for order history
  - products(category_id) for category filtering
```

---

#### ui-ux-designer
**Description**: UI/UX design, wireframes, prototypes, design systems

**Use Cases**:
- Create wireframes for new feature
- Design user flows
- Develop design system

**Example**:
```
Request: "Design login page UI"

Output:
┌─────────────────────────────────────┐
│         MyApp Login                 │
├─────────────────────────────────────┤
│                                     │
│     ┌───────────────────────────┐   │
│     │  Email                    │   │
│     │  user@example.com         │   │
│     └───────────────────────────┘   │
│                                     │
│     ┌───────────────────────────┐   │
│     │  Password                 │   │
│     │  ••••••••                 │   │
│     └───────────────────────────┘   │
│                                     │
│     [ Forgot password? ]            │
│                                     │
│     ┌───────────────────────────┐   │
│     │      LOGIN               │   │
│     └───────────────────────────┘   │
│                                     │
│     Don't have an account? Sign up  │
└─────────────────────────────────────┘

Design Principles:
- Mobile-first responsive
- Accessibility: WCAG AA compliant
- Password visibility toggle
- Clear error messages
```

---

#### security-architect
**Description**: Security design, threat modeling, zero-trust architecture

**Use Cases**:
- Design authentication/authorization system
- Threat modeling for new feature
- Zero-trust architecture

**Example**:
```
Request: "Design security architecture for payment processing"

Output:
Security Architecture:

1. Authentication:
   - OAuth 2.0 + OpenID Connect
   - Multi-factor authentication (MFA) required
   - JWT with short expiry (15 min access, 7 day refresh)

2. Authorization:
   - Role-based access control (RBAC)
   - Least privilege principle
   - Separation of duties

3. Data Protection:
   - Payment data: PCI-DSS compliant
   - Encryption at rest: AES-256
   - Encryption in transit: TLS 1.3
   - Tokenization for credit card numbers

4. Threat Mitigation:
   - SQL Injection: Parameterized queries
   - XSS: Content Security Policy (CSP)
   - CSRF: SameSite cookies + CSRF tokens
   - DDoS: Rate limiting (100 req/min per IP)

5. Monitoring:
   - Security Information and Event Management (SIEM)
   - Intrusion detection system (IDS)
   - Audit logs for all payment transactions
```

---

### 4.2 Implementation (9 Agents)

#### backend-developer
**Description**: Backend implementation, APIs, authentication, business logic

**Most Used Agent**: Handles most backend tasks

**Use Cases**:
- Implement REST/GraphQL API
- User authentication (JWT, OAuth)
- Business logic implementation

**Example**:
```
Request: "Implement user registration API with email validation"

Output:
// src/routes/auth.ts
import { Router } from 'express';
import bcrypt from 'bcrypt';
import { z } from 'zod';
import { prisma } from '../db';
import { sendVerificationEmail } from '../services/email';

const router = Router();

const registerSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
  name: z.string().min(2),
});

router.post('/register', async (req, res) => {
  try {
    // Validate input
    const { email, password, name } = registerSchema.parse(req.body);

    // Check if user exists
    const existing = await prisma.user.findUnique({ where: { email } });
    if (existing) {
      return res.status(409).json({ error: 'Email already registered' });
    }

    // Hash password
    const passwordHash = await bcrypt.hash(password, 10);

    // Create user
    const user = await prisma.user.create({
      data: {
        email,
        passwordHash,
        name,
        verified: false,
      },
    });

    // Send verification email
    const verificationToken = generateToken(user.id);
    await sendVerificationEmail(email, verificationToken);

    res.status(201).json({
      message: 'Registration successful. Please check your email.',
      userId: user.id,
    });
  } catch (error) {
    if (error instanceof z.ZodError) {
      return res.status(400).json({ errors: error.errors });
    }
    console.error('Registration error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

export default router;
```

---

#### frontend-developer
**Description**: Frontend implementation, React/Vue/Angular, state management

**Use Cases**:
- Implement React components
- State management (Redux, Zustand)
- Form handling and validation

**Example**:
```
Request: "Implement login form component in React with TypeScript"

Output:
// src/components/LoginForm.tsx
import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useAuth } from '../hooks/useAuth';

const loginSchema = z.object({
  email: z.string().email('Invalid email'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
});

type LoginFormData = z.infer<typeof loginSchema>;

export const LoginForm: React.FC = () => {
  const { login, isLoading } = useAuth();
  const [error, setError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
  });

  const onSubmit = async (data: LoginFormData) => {
    setError(null);
    try {
      await login(data.email, data.password);
    } catch (err) {
      setError(err.message || 'Login failed');
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="login-form">
      <div className="form-group">
        <label htmlFor="email">Email</label>
        <input
          {...register('email')}
          type="email"
          id="email"
          autoComplete="email"
        />
        {errors.email && <span className="error">{errors.email.message}</span>}
      </div>

      <div className="form-group">
        <label htmlFor="password">Password</label>
        <input
          {...register('password')}
          type="password"
          id="password"
          autoComplete="current-password"
        />
        {errors.password && <span className="error">{errors.password.message}</span>}
      </div>

      {error && <div className="alert alert-error">{error}</div>}

      <button type="submit" disabled={isLoading}>
        {isLoading ? 'Logging in...' : 'Login'}
      </button>
    </form>
  );
};
```

---

#### test-generator
**Description**: Generate Unit/Integration/E2E tests

**Use Cases**:
- Generate unit tests for functions/classes
- Create integration tests for APIs
- E2E tests for user flows

**Example**:
```
Request: "Generate unit tests for UserService"

Output:
// tests/user-service.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { UserService } from '../src/services/user-service';
import { prisma } from '../src/db';

vi.mock('../src/db');

describe('UserService', () => {
  let userService: UserService;

  beforeEach(() => {
    userService = new UserService();
    vi.clearAllMocks();
  });

  describe('createUser', () => {
    it('should create a new user with hashed password', async () => {
      const mockUser = {
        id: '123',
        email: 'test@example.com',
        name: 'Test User',
      };

      (prisma.user.create as any).mockResolvedValue(mockUser);

      const result = await userService.createUser({
        email: 'test@example.com',
        password: 'password123',
        name: 'Test User',
      });

      expect(result).toEqual(mockUser);
      expect(prisma.user.create).toHaveBeenCalledWith({
        data: expect.objectContaining({
          email: 'test@example.com',
          name: 'Test User',
          passwordHash: expect.stringMatching(/^\$2[aby]\$/), // bcrypt hash
        }),
      });
    });

    it('should throw error if email already exists', async () => {
      (prisma.user.create as any).mockRejectedValue({
        code: 'P2002', // Prisma unique constraint error
      });

      await expect(
        userService.createUser({
          email: 'test@example.com',
          password: 'password123',
          name: 'Test User',
        })
      ).rejects.toThrow('Email already exists');
    });

    it('should validate email format', async () => {
      await expect(
        userService.createUser({
          email: 'invalid-email',
          password: 'password123',
          name: 'Test User',
        })
      ).rejects.toThrow('Invalid email format');
    });

    it('should validate password length', async () => {
      await expect(
        userService.createUser({
          email: 'test@example.com',
          password: 'short',
          name: 'Test User',
        })
      ).rejects.toThrow('Password must be at least 8 characters');
    });
  });

  // ... more test suites
});

// Coverage: 92%
```

---

### 4.3 Quality Assurance (11 Agents)

#### code-reviewer
**Description**: Code review, quality scoring (0-100), SOLID principles

**Output**: Detailed review with quality score, strengths, issues, recommendations

**Example**:
```
Request: "Review this UserService class"

Output:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CODE REVIEW REPORT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
File: src/services/user-service.ts
Quality Score: 78/100

✓ STRENGTHS:
  1. Good TypeScript types (no 'any')
  2. Proper error handling with try-catch
  3. Input validation using Zod
  4. Clear function names
  5. Dependency injection for database

⚠ ISSUES:

CRITICAL (0):
  None

HIGH (2):
  1. Line 42: SQL injection risk
     > await db.query(`SELECT * FROM users WHERE email = '${email}'`)
     Fix: Use parameterized queries
     > await db.query('SELECT * FROM users WHERE email = $1', [email])

  2. Line 67: Sensitive data logged
     > console.log('User password:', password)
     Fix: Remove or redact sensitive data in logs

MEDIUM (3):
  1. Line 23: Missing input validation
     Function 'updateUser' doesn't validate input
     Fix: Add Zod schema validation

  2. Line 89: High cyclomatic complexity (15)
     Function 'validateUser' has too many branches
     Fix: Extract validation rules to separate functions

  3. Line 102: Inconsistent error handling
     Some errors throw, others return null
     Fix: Standardize error handling strategy

LOW (4):
  1. Line 12: Magic number
     > if (retries > 3)
     Fix: Extract to constant: MAX_RETRIES = 3

  2. Line 56: Missing JSDoc comment
  3. Line 78: Unused import 'fs'
  4. Line 91: Variable name 'tmp' not descriptive

💡 RECOMMENDATIONS:

Architecture:
  1. Separate concerns: Move validation to separate validator class
  2. Consider adding caching layer for frequently accessed users
  3. Implement repository pattern for database access

Performance:
  1. Add database indexes on 'email' and 'created_at' columns
  2. Use connection pooling to reduce database overhead

Security:
  1. Implement rate limiting for user creation
  2. Add password complexity requirements
  3. Hash passwords with bcrypt (cost factor 12)

Testing:
  1. Current coverage: 65% - Target: 80%+
  2. Missing edge case tests:
     - Empty email
     - Very long passwords (> 100 chars)
     - Concurrent user creation

SOLID Principles:
  ✓ Single Responsibility: Mostly followed
  ✓ Open/Closed: Good use of interfaces
  ✗ Liskov Substitution: N/A
  ⚠ Interface Segregation: Some methods not used by all clients
  ✓ Dependency Inversion: Good use of DI

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SUMMARY:
  - Address 2 high-priority security issues immediately
  - Refactor complex validation function
  - Add missing tests to reach 80% coverage
  - Overall good code quality, needs minor improvements
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

#### security-tester
**Description**: Security testing, OWASP Top 10, vulnerability scanning

**Use Cases**:
- Test for SQL injection, XSS, CSRF
- Validate authentication/authorization
- Check for common vulnerabilities

**Example**:
```
Request: "Security test the user authentication API"

Output:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SECURITY TEST REPORT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Target: /api/auth/* endpoints
Date: 2025-01-06
Duration: 3m 42s

OWASP TOP 10 TESTING:

[1] Broken Access Control: ✓ PASS
  - Tested unauthorized access to protected endpoints
  - JWT validation working correctly
  - Role-based access control enforced

[2] Cryptographic Failures: ⚠ WARNING
  ✓ Passwords hashed with bcrypt (cost 10)
  ⚠ JWT secret stored in code (should be env var)
  ⚠ No HTTPS enforcement detected

[3] Injection: ❌ CRITICAL
  ❌ SQL Injection vulnerability in /api/auth/login
     Payload: email=' OR '1'='1
     Result: Bypassed authentication!
     Fix: Use parameterized queries

[4] Insecure Design: ✓ PASS
  - Rate limiting in place (10 req/min)
  - Account lockout after 5 failed attempts
  - Password complexity enforced

[5] Security Misconfiguration: ⚠ WARNING
  ⚠ Debug mode enabled in production
  ⚠ Detailed error messages exposed
  ⚠ CORS allowing all origins (*)

[6] Vulnerable Components: ✓ PASS
  - All dependencies up to date
  - No known CVEs in npm packages

[7] Authentication Failures: ⚠ WARNING
  ✓ Password hashing secure
  ⚠ No multi-factor authentication (MFA)
  ⚠ Session timeout too long (24 hours)

[8] Data Integrity Failures: ✓ PASS
  - JWT signatures validated
  - Integrity checks in place

[9] Logging & Monitoring Failures: ⚠ WARNING
  ⚠ No logging for failed login attempts
  ⚠ No alerts for suspicious activity

[10] Server-Side Request Forgery: ✓ PASS
  - No SSRF vectors detected

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
VULNERABILITY SUMMARY:

CRITICAL (1):
  🔴 SQL Injection in /api/auth/login
     Impact: Complete authentication bypass
     Priority: Fix immediately

HIGH (0):
  None

MEDIUM (4):
  🟡 JWT secret hardcoded in source
  🟡 No HTTPS enforcement
  🟡 CORS misconfiguration
  🟡 Debug mode in production

LOW (3):
  🟢 No MFA support
  🟢 Long session timeout
  🟢 Missing audit logs

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
IMMEDIATE ACTIONS REQUIRED:

1. Fix SQL injection (CRITICAL)
   File: src/routes/auth.ts, line 42
   Before: db.query(`SELECT * FROM users WHERE email = '${email}'`)
   After:  db.query('SELECT * FROM users WHERE email = $1', [email])

2. Move JWT secret to environment variable
   File: src/config.ts, line 12
   Add to .env: JWT_SECRET=<random_256_bit_key>

3. Enable HTTPS enforcement
   Add helmet middleware:
   app.use(helmet.hsts({ maxAge: 31536000 }))

4. Restrict CORS
   Update CORS config:
   cors({ origin: process.env.ALLOWED_ORIGINS.split(',') })

5. Disable debug mode in production
   Set NODE_ENV=production

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RECOMMENDED ENHANCEMENTS:

- Implement MFA (Google Authenticator, SMS)
- Reduce session timeout to 15 minutes
- Add audit logging for all authentication events
- Implement Web Application Firewall (WAF)
- Set up security monitoring and alerts

SECURITY SCORE: 62/100
  (Score will increase to 85/100 after fixing critical/high issues)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

### 4.4 Operations (10 Agents)

#### devops-engineer
**Description**: DevOps, IaC (Terraform), Kubernetes, infrastructure management

**Use Cases**:
- Infrastructure as Code (Terraform, Pulumi)
- Kubernetes manifests
- CI/CD pipelines

**Example**:
```
Request: "Create Terraform configuration for AWS deployment"

Output:
# infrastructure/main.tf
terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket = "myapp-terraform-state"
    key    = "production/terraform.tfstate"
    region = "us-east-1"
  }
}

provider "aws" {
  region = var.aws_region
}

# VPC
resource "aws_vpc" "main" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name        = "${var.project_name}-vpc"
    Environment = var.environment
  }
}

# Public Subnets
resource "aws_subnet" "public" {
  count                   = 2
  vpc_id                  = aws_vpc.main.id
  cidr_block              = "10.0.${count.index}.0/24"
  availability_zone       = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name = "${var.project_name}-public-${count.index + 1}"
  }
}

# ECS Cluster
resource "aws_ecs_cluster" "main" {
  name = "${var.project_name}-cluster"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

# Application Load Balancer
resource "aws_lb" "main" {
  name               = "${var.project_name}-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = aws_subnet.public[*].id

  enable_deletion_protection = var.environment == "production"
}

# RDS PostgreSQL
resource "aws_db_instance" "main" {
  identifier        = "${var.project_name}-db"
  engine            = "postgres"
  engine_version    = "15.3"
  instance_class    = var.db_instance_class
  allocated_storage = 20
  storage_encrypted = true

  db_name  = var.db_name
  username = var.db_username
  password = var.db_password

  vpc_security_group_ids = [aws_security_group.db.id]
  db_subnet_group_name   = aws_db_subnet_group.main.name

  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "Mon:04:00-Mon:05:00"

  skip_final_snapshot = var.environment != "production"
}

# Outputs
output "alb_dns_name" {
  value = aws_lb.main.dns_name
}

output "db_endpoint" {
  value     = aws_db_instance.main.endpoint
  sensitive = true
}
```

---

#### cicd-manager
**Description**: CI/CD pipelines, GitHub Actions, quality gates, deployment automation

**Use Cases**:
- Create GitHub Actions workflows
- Set up quality gates
- Deployment automation

**Example**:
```
Request: "Create CI/CD pipeline for Node.js app with quality gates"

Output:
# .github/workflows/ci-cd.yml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  NODE_VERSION: '20'

jobs:
  quality-checks:
    name: Code Quality & Security
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Lint
        run: npm run lint

      - name: Format check
        run: npm run format:check

      - name: Type check
        run: npm run type-check

      - name: Security audit
        run: npm audit --audit-level=high

      - name: SAST scan
        uses: github/codeql-action/analyze@v2

  test:
    name: Tests
    runs-on: ubuntu-latest
    needs: quality-checks
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Unit tests
        run: npm run test:unit

      - name: Integration tests
        run: npm run test:integration

      - name: E2E tests
        run: npm run test:e2e

      - name: Generate coverage
        run: npm run test:coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3

      - name: Quality Gate - Coverage
        run: |
          COVERAGE=$(npm run test:coverage:json | jq '.total.lines.pct')
          if (( $(echo "$COVERAGE < 80" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 80% threshold"
            exit 1
          fi

  build:
    name: Build
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Build
        run: npm run build

      - name: Upload build artifacts
        uses: actions/upload-artifact@v3
        with:
          name: build
          path: dist/

  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/develop'
    environment:
      name: staging
      url: https://staging.myapp.com
    steps:
      - name: Download build artifacts
        uses: actions/download-artifact@v3
        with:
          name: build

      - name: Deploy to AWS ECS
        run: |
          # Deploy to staging ECS cluster
          aws ecs update-service \
            --cluster staging-cluster \
            --service myapp-service \
            --force-new-deployment

      - name: Smoke tests
        run: |
          curl -f https://staging.myapp.com/health || exit 1

  deploy-production:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/main'
    environment:
      name: production
      url: https://myapp.com
    steps:
      - name: Download build artifacts
        uses: actions/download-artifact@v3
        with:
          name: build

      - name: Deploy to AWS ECS (Blue/Green)
        run: |
          # Blue/Green deployment
          aws ecs update-service \
            --cluster production-cluster \
            --service myapp-service \
            --deployment-configuration "deploymentCircuitBreaker={enable=true,rollback=true}"

      - name: Wait for deployment
        run: |
          aws ecs wait services-stable \
            --cluster production-cluster \
            --services myapp-service

      - name: Smoke tests
        run: |
          curl -f https://myapp.com/health || exit 1

      - name: Create GitHub Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: v${{ github.run_number }}
          release_name: Release v${{ github.run_number }}
          draft: false
          prerelease: false

# Quality Gates Summary:
# ✓ Linting (ESLint)
# ✓ Formatting (Prettier)
# ✓ Type checking (TypeScript)
# ✓ Security audit (npm audit)
# ✓ SAST scanning (CodeQL)
# ✓ Test coverage >= 80%
# ✓ All tests pass
# ✓ Smoke tests pass after deployment
```

---

### 4.5 Meta (11 Agents)

#### metrics-collector
**Description**: Collect DORA metrics, KPIs, dashboards

**Use Cases**:
- Track deployment frequency
- Measure lead time for changes
- Monitor change failure rate
- Calculate MTTR (Mean Time To Recover)

**Example**:
```
Request: "Collect DORA metrics for the past month"

Output:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
DORA METRICS REPORT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Period: 2024-12-06 to 2025-01-06
Team: Backend Engineering

1. DEPLOYMENT FREQUENCY
   Current: 12 deployments/day
   Previous: 8 deployments/day
   Change: +50% ↑
   Level: Elite (>1 per day) ✓

2. LEAD TIME FOR CHANGES
   Current: 2.3 hours
   Previous: 4.1 hours
   Change: -44% ↓
   Level: Elite (<1 day) ✓

3. CHANGE FAILURE RATE
   Current: 8.2%
   Previous: 12.5%
   Change: -34% ↓
   Level: High (target: <5%) ⚠

4. TIME TO RESTORE SERVICE
   Current: 28 minutes
   Previous: 45 minutes
   Change: -38% ↓
   Level: Elite (<1 hour) ✓

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OVERALL DORA LEVEL: HIGH

Elite: 3/4 metrics
High: 1/4 metrics

Recommendation:
  - Focus on reducing change failure rate to <5%
  - Investigate root causes of recent failures
  - Consider adding more automated testing
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Additional KPIs:

Code Quality:
  - Test coverage: 87% (target: 80%) ✓
  - Code review time: 3.2 hours (avg)
  - Pull request size: 142 lines (avg)

Performance:
  - API latency (P95): 245ms
  - Error rate: 0.3%
  - Uptime: 99.95%

Team Velocity:
  - Story points completed: 142 (2-week sprint)
  - Velocity trend: +12% over 3 sprints
  - Bug fixing time: 1.8 days (avg)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 5. Usage Patterns

### Pattern 1: Full Development Cycle

```
1. Planning: system-architect + api-designer + database-designer (parallel)
2. Implementation: backend-developer + frontend-developer (parallel)
3. QA: test-generator + code-reviewer + security-tester (parallel)
4. Deployment: cicd-manager
5. Monitoring: monitoring-specialist
6. Metrics: metrics-collector
```

**Coordinator Request**:
```
"Develop a complete user management feature from design to deployment"
```

---

### Pattern 2: Bug Fix Workflow

```
1. Diagnosis: bug-fixer (root cause analysis)
2. Fix: backend-developer or frontend-developer
3. Test: test-generator (regression tests)
4. Review: code-reviewer
5. Deploy: cicd-manager
```

**Coordinator Request**:
```
"Fix the login timeout bug, add tests, and deploy"
```

---

### Pattern 3: Performance Optimization

```
1. Profiling: performance-tester (identify bottlenecks)
2. Analysis: complexity-analyzer
3. Refactoring: refactor-specialist
4. Validation: performance-tester (before/after comparison)
5. Documentation: tech-writer
```

**Coordinator Request**:
```
"Optimize the search API performance"
```

---

### Pattern 4: Security Audit

```
1. Architecture Review: security-architect
2. Code Scan: security-scanner (SAST/DAST)
3. Penetration Test: security-tester
4. Fixes: bug-fixer
5. Validation: security-scanner (re-scan)
```

**Coordinator Request**:
```
"Perform complete security audit and fix vulnerabilities"
```

---

## 6. Parallel Execution with Tmux

### What is Tmux Integration?

Tmux allows running multiple agents in **isolated sessions**, enabling:
- Parallel execution (2-5 agents simultaneously)
- Real-time output monitoring
- Session persistence (survives terminal disconnect)
- Interactive debugging

### When to Use Tmux?

**Auto-triggered** by Coordinator when:
1. **Parallel execution**: 2+ agents
2. **Long-running tasks**: Build, test, deploy
3. **Debug-required tasks**: Incident response, performance investigation

**Manual trigger**:
```
Ctrl+P → "Run with Tmux"
Or
Include "tmux" in your request: "Use tmux to run api-designer and database-designer in parallel"
```

### Tmux Commands

```bash
# View all sessions
Ctrl+T (in editor)
Or
tmux list-sessions

# Attach to specific session
tmux attach -t ait42-backend-dev-1234

# Kill session
:tmux-kill 1234 (in editor)
Or
tmux kill-session -t ait42-backend-dev-1234
```

### Example: Parallel Design

```
Request: "Design e-commerce system with API and database"

→ Coordinator creates 2 Tmux sessions:
  1. ait42-api-designer-1234
  2. ait42-database-designer-1235

→ Both agents execute in parallel

→ View output in editor: Ctrl+T
```

---

## 7. Best Practices

### 1. Use Coordinator for Unclear Tasks

```
❌ Bad: Manually selecting agents when you're not sure
✓ Good: "I need to improve application security" → Coordinator selects optimal agents
```

### 2. Provide Context

```
❌ Bad: "Implement user feature"
✓ Good: "Implement user authentication API with JWT, email validation, and PostgreSQL storage"
```

### 3. Review Agent Output

Always review generated code:
- Check for security issues
- Verify business logic
- Ensure coding standards compliance

### 4. Combine Multiple Agents

```
Example: "Generate code, then review it, then create tests"
→ Coordinator runs: backend-developer → code-reviewer → test-generator (sequential)
```

### 5. Use Specific Agents for Specialized Tasks

```
For deep security analysis: security-architect + security-tester + security-scanner
For performance optimization: performance-tester + refactor-specialist + complexity-analyzer
```

---

## 8. Troubleshooting

### Agent Not Found

**Problem**: "Agent 'xyz' not found"

**Solution**:
```bash
# Verify AIT42_ROOT is set
echo $AIT42_ROOT

# Check agent files exist
ls $AIT42_ROOT/.claude/agents/ | grep xyz

# Reload agents in editor
:reload-agents
```

---

### Agent Execution Timeout

**Problem**: Agent times out after 5 minutes

**Solution**:
```toml
# ~/.config/ait42-editor/config.toml
[ait42]
session_timeout = 600  # Increase to 10 minutes
```

---

### Tmux Session Not Starting

**Problem**: "Failed to create tmux session"

**Solution**:
```bash
# Check tmux is installed
which tmux

# Install if missing
brew install tmux

# Verify tmux works
tmux new-session -d -s test
tmux kill-session -t test
```

---

### Poor Agent Output Quality

**Problem**: Generated code doesn't meet expectations

**Solution**:
1. **Provide more context**: Include tech stack, existing code patterns
2. **Use code-reviewer**: Review output and request improvements
3. **Iterate**: "Improve this code to use TypeScript strict mode"

---

## Summary

### 49 Agents at Your Fingertips

- **Planning & Design**: 8 agents for architecture, API, database, UI, security
- **Implementation**: 9 agents for backend, frontend, testing, scripts
- **Quality Assurance**: 11 agents for code review, testing, refactoring, security
- **Operations**: 10 agents for DevOps, CI/CD, monitoring, infrastructure
- **Meta**: 11 agents for process optimization, metrics, documentation, learning

### How to Get Started

1. **Set AIT42_ROOT**: `export AIT42_ROOT=/path/to/AIT42`
2. **Open editor**: `ait42-editor`
3. **Press Ctrl+P**: Open command palette
4. **Describe task**: Coordinator auto-selects agents
5. **Review results**: Agent output in status bar

### Key Features

- **Coordinator**: Auto-selects optimal agents (no need to memorize 49 names!)
- **Parallel Execution**: Run 2-5 agents simultaneously via Tmux
- **Quality Assurance**: Built-in code review, security testing, coverage validation
- **Full Lifecycle**: From planning to deployment to metrics

### Support

- Documentation: This guide + USER_GUIDE.md + DEVELOPER_GUIDE.md
- Issues: https://github.com/RenTonoduka/AIT42/issues
- Agent Definitions: $AIT42_ROOT/.claude/agents/*.md

---

**End of Agent Integration Guide**

**License**: MIT
