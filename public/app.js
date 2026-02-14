// Configuration
const API_URL = window.location.hostname === 'localhost'
    ? 'http://localhost:3000'
    : 'https://api.policycheck.openattribution.org';

let currentResults = null;

// DOM Elements
const urlInput = document.getElementById('url-input');
const analyzeBtn = document.getElementById('analyze-btn');
const csvFile = document.getElementById('csv-file');
const analyzeCsvBtn = document.getElementById('analyze-csv-btn');
const uploadArea = document.getElementById('upload-area');
const loading = document.getElementById('loading');
const results = document.getElementById('results');
const error = document.getElementById('error');
const errorMessage = document.getElementById('error-message');
const resultsBody = document.getElementById('results-body');
const summary = document.getElementById('summary');
const downloadJsonBtn = document.getElementById('download-json');
const downloadCsvBtn = document.getElementById('download-csv');

// Tab switching
document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
        const tabName = tab.dataset.tab;

        // Update tabs
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');

        // Update content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });
        document.getElementById(`${tabName}-tab`).classList.add('active');

        // Reset states
        hideAll();
    });
});

// Single URL Analysis
analyzeBtn.addEventListener('click', async () => {
    const url = urlInput.value.trim();
    if (!url) {
        showError('Please enter a URL');
        return;
    }

    await analyzeUrls([url]);
});

// Handle Enter key in URL input
urlInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        analyzeBtn.click();
    }
});

// CSV Upload
csvFile.addEventListener('change', (e) => {
    const file = e.target.files[0];
    if (file) {
        analyzeCsvBtn.disabled = false;
        uploadArea.querySelector('.upload-content p').textContent = `Selected: ${file.name}`;
    }
});

// Drag and drop
uploadArea.addEventListener('dragover', (e) => {
    e.preventDefault();
    uploadArea.classList.add('drag-over');
});

uploadArea.addEventListener('dragleave', () => {
    uploadArea.classList.remove('drag-over');
});

uploadArea.addEventListener('drop', (e) => {
    e.preventDefault();
    uploadArea.classList.remove('drag-over');

    const file = e.dataTransfer.files[0];
    if (file && file.name.endsWith('.csv')) {
        csvFile.files = e.dataTransfer.files;
        csvFile.dispatchEvent(new Event('change'));
    } else {
        showError('Please drop a CSV file');
    }
});

uploadArea.addEventListener('click', () => {
    csvFile.click();
});

// CSV Analysis
analyzeCsvBtn.addEventListener('click', async () => {
    const file = csvFile.files[0];
    if (!file) return;

    try {
        const text = await file.text();
        const urls = parseCSV(text);

        if (urls.length === 0) {
            showError('No URLs found in CSV. Make sure there is a "url" column.');
            return;
        }

        await analyzeUrls(urls);
    } catch (err) {
        showError(`Failed to parse CSV: ${err.message}`);
    }
});

// Parse CSV
function parseCSV(text) {
    const lines = text.trim().split('\n');
    if (lines.length === 0) return [];

    const headers = lines[0].split(',').map(h => h.trim().toLowerCase());
    const urlIndex = headers.findIndex(h =>
        h.includes('url') || h === 'link' || h === 'website'
    );

    if (urlIndex === -1) {
        // No header found, assume first column
        return lines.slice(1)
            .map(line => line.split(',')[0].trim())
            .filter(url => url && url.length > 0);
    }

    return lines.slice(1)
        .map(line => {
            const cols = line.split(',');
            return cols[urlIndex]?.trim();
        })
        .filter(url => url && url.length > 0);
}

// Main analysis function
async function analyzeUrls(urls) {
    hideAll();
    loading.classList.remove('hidden');

    try {
        const response = await fetch(`${API_URL}/analyze`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                urls: urls,
                user_agent: '*'
            })
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();
        currentResults = data.results;

        displayResults(data);
    } catch (err) {
        showError(`Analysis failed: ${err.message}`);
    } finally {
        loading.classList.add('hidden');
    }
}

// Display results
function displayResults(data) {
    // Show results section
    results.classList.remove('hidden');

    // Update summary
    summary.innerHTML = `
        <strong>Total:</strong> ${data.total} &nbsp;|&nbsp;
        <strong class="status-success">✓ Successful:</strong> ${data.successful} &nbsp;|&nbsp;
        <strong class="status-error">✗ Failed:</strong> ${data.failed}
    `;

    // Clear existing results
    resultsBody.innerHTML = '';

    // Add rows
    data.results.forEach(result => {
        const row = document.createElement('tr');

        // Status
        const statusClass = result.status === 'success' ? 'status-success' : 'status-error';
        const statusText = result.status === 'success' ? '✓ Success' : '✗ Error';

        // Path allowed
        let pathAllowed = '-';
        if (result.status === 'success') {
            pathAllowed = result.is_path_allowed
                ? '<span class="status-yes">✓ Yes</span>'
                : '<span class="status-no">✗ No</span>';
        }

        // RSL Licenses
        const rslCount = result.active_licenses?.length || 0;
        const rslText = rslCount > 0 ? `${rslCount} license(s)` : '-';

        // TDM
        let tdmText = '-';
        if (result.tdm_policy) {
            tdmText = result.tdm_policy.is_reserved
                ? '<span class="status-warning">⚠️ Yes</span>'
                : '<span class="status-yes">✓ No</span>';
        }

        // User agents
        const userAgents = result.user_agents?.slice(0, 3).join(', ') || '-';
        const moreAgents = result.user_agents?.length > 3 ? ` (+${result.user_agents.length - 3})` : '';

        // Crawl delay
        const crawlDelay = result.crawl_delay ? `${result.crawl_delay}s` : '-';

        row.innerHTML = `
            <td class="url-cell" title="${result.url}">${result.url}</td>
            <td class="${statusClass}">${statusText}</td>
            <td>${pathAllowed}</td>
            <td>${rslText}</td>
            <td>${tdmText}</td>
            <td>${userAgents}${moreAgents}</td>
            <td>${crawlDelay}</td>
        `;

        resultsBody.appendChild(row);
    });
}

// Download handlers
downloadJsonBtn.addEventListener('click', () => {
    if (!currentResults) return;

    const blob = new Blob([JSON.stringify(currentResults, null, 2)], { type: 'application/json' });
    downloadFile(blob, 'policycheck-results.json');
});

downloadCsvBtn.addEventListener('click', () => {
    if (!currentResults) return;

    // Build CSV
    const headers = ['URL', 'Status', 'Path Allowed', 'RSL Licenses', 'TDM Reserved', 'User Agents', 'Crawl Delay'];
    const rows = currentResults.map(r => [
        r.url,
        r.status,
        r.is_path_allowed ? 'Yes' : 'No',
        r.active_licenses?.length || 0,
        r.tdm_policy?.is_reserved ? 'Yes' : 'No',
        r.user_agents?.join('; ') || '',
        r.crawl_delay || ''
    ]);

    const csv = [headers, ...rows].map(row => row.join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    downloadFile(blob, 'policycheck-results.csv');
});

function downloadFile(blob, filename) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
}

// Helper functions
function hideAll() {
    results.classList.add('hidden');
    error.classList.add('hidden');
    loading.classList.add('hidden');
}

function showError(msg) {
    hideAll();
    error.classList.remove('hidden');
    errorMessage.textContent = msg;
}

// Initialize
console.log('PolicyCheck Web UI initialized');
console.log('API URL:', API_URL);
