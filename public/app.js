// Configuration
// Update this URL once the backend is deployed to Fly.io
const API_URL = 'http://localhost:3000';
// Production: 'https://api.policycheck.openattribution.org';

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
document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        const tabName = btn.dataset.tab;

        // Update buttons
        document.querySelectorAll('.tab-btn').forEach(b => {
            b.classList.remove('active', 'border-coral-600', 'text-gray-900', 'font-normal');
            b.classList.add('border-transparent', 'text-gray-600', 'font-light');
        });
        btn.classList.add('active', 'border-coral-600', 'text-gray-900', 'font-normal');
        btn.classList.remove('border-transparent', 'text-gray-600', 'font-light');

        // Update content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.add('hidden');
        });
        document.getElementById(`${tabName}-tab`).classList.remove('hidden');

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

// Handle Enter key
urlInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') analyzeBtn.click();
});

// CSV Upload
csvFile.addEventListener('change', (e) => {
    const file = e.target.files[0];
    if (file) {
        analyzeCsvBtn.disabled = false;
        uploadArea.querySelector('p').innerHTML = `Selected: <span class="font-normal">${file.name}</span>`;
    }
});

// Drag and drop
uploadArea.addEventListener('dragover', (e) => {
    e.preventDefault();
    uploadArea.classList.add('border-coral-600', 'bg-coral-50');
});

uploadArea.addEventListener('dragleave', () => {
    uploadArea.classList.remove('border-coral-600', 'bg-coral-50');
});

uploadArea.addEventListener('drop', (e) => {
    e.preventDefault();
    uploadArea.classList.remove('border-coral-600', 'bg-coral-50');

    const file = e.dataTransfer.files[0];
    if (file && file.name.endsWith('.csv')) {
        csvFile.files = e.dataTransfer.files;
        csvFile.dispatchEvent(new Event('change'));
    } else {
        showError('Please drop a CSV file');
    }
});

uploadArea.addEventListener('click', () => csvFile.click());

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
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ urls, user_agent: '*' })
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
    results.classList.remove('hidden');

    summary.innerHTML = `
        <strong>Total:</strong> ${data.total} &nbsp;|&nbsp;
        <strong class="text-green-700">✓ Successful:</strong> ${data.successful} &nbsp;|&nbsp;
        <strong class="text-coral-700">✗ Failed:</strong> ${data.failed}
    `;

    resultsBody.innerHTML = '';

    data.results.forEach(result => {
        const row = document.createElement('tr');
        row.className = 'border-b border-gray-100 hover:bg-gray-50';

        const statusClass = result.status === 'success' ? 'text-green-700' : 'text-coral-700';
        const statusText = result.status === 'success' ? '✓ Success' : '✗ Error';

        let pathAllowed = '-';
        if (result.status === 'success') {
            pathAllowed = result.is_path_allowed
                ? '<span class="text-green-700">✓ Yes</span>'
                : '<span class="text-coral-700">✗ No</span>';
        }

        const rslCount = result.active_licenses?.length || 0;
        const rslText = rslCount > 0 ? `${rslCount} license(s)` : '-';

        let tdmText = '-';
        if (result.tdm_policy) {
            tdmText = result.tdm_policy.is_reserved
                ? '<span class="text-amber-700">⚠️ Yes</span>'
                : '<span class="text-green-700">✓ No</span>';
        }

        const userAgents = result.user_agents?.slice(0, 2).join(', ') || '-';
        const moreAgents = result.user_agents?.length > 2 ? ` (+${result.user_agents.length - 2})` : '';

        row.innerHTML = `
            <td class="py-3 px-4 max-w-xs truncate" title="${result.url}">${result.url}</td>
            <td class="py-3 px-4 ${statusClass}">${statusText}</td>
            <td class="py-3 px-4">${pathAllowed}</td>
            <td class="py-3 px-4">${rslText}</td>
            <td class="py-3 px-4">${tdmText}</td>
            <td class="py-3 px-4">${userAgents}${moreAgents}</td>
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

    const headers = ['URL', 'Status', 'Path Allowed', 'RSL Licenses', 'TDM Reserved', 'User Agents'];
    const rows = currentResults.map(r => [
        r.url,
        r.status,
        r.is_path_allowed ? 'Yes' : 'No',
        r.active_licenses?.length || 0,
        r.tdm_policy?.is_reserved ? 'Yes' : 'No',
        r.user_agents?.join('; ') || ''
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
