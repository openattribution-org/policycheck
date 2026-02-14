# Deployment Guide

This guide explains how to deploy PolicyCheck as a free web service with:
- **Backend API**: Fly.io (free tier)
- **Frontend UI**: Cloudflare Pages (free)
- **Domain**: policycheck.openattribution.org

## Prerequisites

- [Fly.io CLI](https://fly.io/docs/hands-on/install-flyctl/) installed
- Cloudflare account with access to openattribution.org domain
- Git repository access

## Backend Deployment (Fly.io)

### 1. Install Fly CLI

```bash
# macOS/Linux
curl -L https://fly.io/install.sh | sh

# Or use Homebrew
brew install flyctl
```

### 2. Login to Fly.io

```bash
flyctl auth login
```

### 3. Create and Deploy the App

```bash
# Create the app (first time only)
flyctl apps create policycheck-api

# Deploy
flyctl deploy

# This will:
# - Build the Docker image
# - Push to Fly.io registry
# - Deploy to free tier VM
# - Allocate a fly.dev subdomain
```

### 4. Check Status

```bash
# View app status
flyctl status

# View logs
flyctl logs

# Open in browser
flyctl open
```

### 5. Custom Domain Setup

Set up `api.policycheck.openattribution.org`:

```bash
# Add certificate
flyctl certs add api.policycheck.openattribution.org

# Fly will provide DNS records to add:
# - CNAME: api.policycheck.openattribution.org -> <app-name>.fly.dev
```

Add the CNAME record in your DNS provider (Cloudflare):
```
Type: CNAME
Name: api.policycheck
Content: policycheck-api.fly.dev
Proxy: Yes (orange cloud)
```

### 6. Verify Deployment

```bash
curl https://api.policycheck.openattribution.org/health

# Expected response:
# {"status":"healthy","service":"policycheck","version":"0.1.0"}
```

## Frontend Deployment (Cloudflare Pages)

### Option 1: Via Cloudflare Dashboard (Recommended)

1. **Go to Cloudflare Pages**
   - Visit https://dash.cloudflare.com
   - Select your account
   - Go to "Workers & Pages" → "Create application" → "Pages"

2. **Connect Repository**
   - Connect to GitHub
   - Select `openattribution-org/policycheck`

3. **Configure Build Settings**
   ```
   Build command: cd public && npm install && npm run build:css
   Build output directory: /public
   Root directory: (leave empty)
   Environment variables: (none needed)
   ```

4. **Deploy**
   - Click "Save and Deploy"
   - Cloudflare will deploy the `/public` directory

5. **Custom Domain**
   - Go to "Custom domains"
   - Add `policycheck.openattribution.org`
   - Cloudflare will automatically configure DNS

### Option 2: Via Wrangler CLI

```bash
# Install Wrangler
npm install -g wrangler

# Login
wrangler login

# Deploy
wrangler pages deploy public --project-name=policycheck

# Set custom domain
wrangler pages domains add policycheck.openattribution.org
```

## Environment Configuration

The frontend automatically detects the environment:
- **Local development**: `http://localhost:3000`
- **Production**: `https://api.policycheck.openattribution.org`

No environment variables needed!

## DNS Configuration Summary

Add these records to openattribution.org in Cloudflare DNS:

```
# Backend API
Type: CNAME
Name: api.policycheck
Content: policycheck-api.fly.dev
Proxy: Yes

# Frontend (auto-configured by Cloudflare Pages)
Type: CNAME
Name: policycheck
Content: <project-name>.pages.dev
Proxy: Yes
```

## Testing the Deployment

### 1. Test API Endpoint

```bash
curl -X POST https://api.policycheck.openattribution.org/analyze \
  -H "Content-Type: application/json" \
  -d '{"urls":["https://github.com"],"user_agent":"*"}'
```

### 2. Test Web UI

Visit: https://policycheck.openattribution.org

- Enter a URL (e.g., `https://github.com`)
- Click "Analyze"
- Verify results display correctly

## Monitoring & Maintenance

### Fly.io Backend

```bash
# View logs
flyctl logs

# SSH into instance
flyctl ssh console

# Scale (if needed, still free tier)
flyctl scale count 1

# Update deployment
git push origin main
flyctl deploy
```

### Cloudflare Pages

- **Auto-deploy**: Cloudflare Pages auto-deploys on every push to `main`
- **Logs**: Available in Cloudflare Dashboard → Pages → Deployment
- **Analytics**: Built-in analytics in Cloudflare Pages

## Cost Breakdown

**Total Cost: $0/month** 🎉

- **Fly.io**: Free tier includes:
  - 3 shared-cpu VMs (using 1)
  - 160GB bandwidth/month
  - Auto-scaling to 0 when idle

- **Cloudflare Pages**: Free tier includes:
  - Unlimited bandwidth
  - Unlimited requests
  - 500 builds/month
  - Built-in CDN

## Troubleshooting

### Backend Issues

**App won't start:**
```bash
flyctl logs
# Check for errors in Rust compilation or runtime
```

**CORS errors:**
- Verify CORS is enabled in `src/server.rs`
- Check that frontend URL is correct in `app.js`

**Out of memory:**
```bash
# Increase VM memory (still free tier)
flyctl scale memory 512
```

### Frontend Issues

**API calls failing:**
- Check browser console for errors
- Verify API_URL in `public/app.js`
- Test API endpoint directly with curl

**Pages not updating:**
- Check deployment status in Cloudflare Dashboard
- Trigger manual redeploy if needed
- Clear browser cache

## Updating the Service

### Backend

```bash
# Make changes
git add .
git commit -m "update: description"
git push origin main

# Deploy to Fly.io
flyctl deploy
```

### Frontend

```bash
# Make changes to public/ directory
git add public/
git commit -m "update: UI improvements"
git push origin main

# Cloudflare Pages auto-deploys!
# No manual deployment needed
```

## Rollback

### Backend

```bash
# List releases
flyctl releases

# Rollback to previous version
flyctl releases rollback <version-number>
```

### Frontend

- Go to Cloudflare Dashboard → Pages → Deployments
- Click "Rollback" on any previous deployment

## Support

- **Fly.io Docs**: https://fly.io/docs
- **Cloudflare Pages Docs**: https://developers.cloudflare.com/pages
- **PolicyCheck Issues**: https://github.com/openattribution-org/policycheck/issues
