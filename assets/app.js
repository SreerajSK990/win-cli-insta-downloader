const urlInput = document.getElementById("urlInput");
const fetchBtn = document.getElementById("fetchBtn");
const cookieInput = document.getElementById("cookieInput");
const toggleCookieBtn = document.getElementById("toggleCookieBtn");
const cookieDrawer = document.getElementById("cookieDrawer");
const statusBanner = document.getElementById("statusBanner");
const statusText = document.getElementById("statusText");
const resultsPanel = document.getElementById("resultsPanel");
const storagePath = document.getElementById("storagePath");
const postOwner = document.getElementById("postOwner");
const postShortcode = document.getElementById("postShortcode");
const postCaption = document.getElementById("postCaption");
const countBadge = document.getElementById("countBadge");
const selectAllBtn = document.getElementById("selectAllBtn");
const deselectAllBtn = document.getElementById("deselectAllBtn");
const downloadSelectedBtn = document.getElementById("downloadSelectedBtn");
const downloadAllBtn = document.getElementById("downloadAllBtn");
const mediaGrid = document.getElementById("mediaGrid");

let currentPost = null;

async function init() {
    try {
        const response = await fetch("/api/info");
        const json = await response.json();
        if (json.success && json.data) {
            storagePath.textContent = `Saving to: ${json.data.download_dir}`;
        }
    } catch (e) {
        storagePath.textContent = "Saving to: Default Downloads";
    }
}

init();

toggleCookieBtn.addEventListener("click", () => {
    cookieDrawer.classList.toggle("hidden");
    if (!cookieDrawer.classList.contains("hidden")) {
        cookieInput.focus();
    }
});

function showStatus(message, type = "normal") {
    statusBanner.className = "status-banner";
    if (type === "error") {
        statusBanner.classList.add("error");
    } else if (type === "success") {
        statusBanner.classList.add("success");
    }
    statusBanner.classList.remove("hidden");
    statusText.textContent = message;
}

function hideStatus() {
    statusBanner.classList.add("hidden");
}

urlInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
        fetchBtn.click();
    }
});

fetchBtn.addEventListener("click", async () => {
    const url = urlInput.value.trim();
    if (!url) {
        showStatus("Please paste a valid Instagram link", "error");
        return;
    }

    const cookie = cookieInput.value.trim() || null;

    showStatus("Fetching media details...", "normal");
    fetchBtn.disabled = true;

    try {
        const response = await fetch("/api/fetch", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ url, cookie }),
        });

        const result = await response.json();

        if (!result.success || !result.data) {
            throw new Error(result.error || "Unable to extract media from this post");
        }

        currentPost = result.data;
        renderPost(currentPost);
        showStatus(`Successfully loaded ${currentPost.items.length} item(s)`, "success");
    } catch (err) {
        showStatus(err.message, "error");
        resultsPanel.classList.add("hidden");
    } finally {
        fetchBtn.disabled = false;
    }
});

function renderPost(post) {
    resultsPanel.classList.remove("hidden");

    postOwner.textContent = post.owner_username ? `@${post.owner_username}` : "Instagram Post";
    postShortcode.textContent = `Shortcode: ${post.shortcode}`;
    postCaption.textContent = post.caption || "No caption provided.";
    countBadge.textContent = `${post.items.len || post.items.length} item(s)`;

    mediaGrid.innerHTML = "";

    post.items.forEach((item, index) => {
        const total = post.items.length;
        const card = document.createElement("div");
        card.className = "media-card";

        const extension = item.media_type === "video" ? "mp4" : "jpg";
        const filename = total <= 1 
            ? `insta_${post.shortcode}.${extension}` 
            : `insta_${post.shortcode}_${String(index + 1).padStart(2, "0")}.${extension}`;

        const isVideo = item.media_type === "video";
        const previewElement = isVideo 
            ? `<video controls playsinline preload="metadata" poster="${item.thumbnail_url}">
                   <source src="${item.url}" type="video/mp4">
               </video>`
            : `<img src="${item.thumbnail_url || item.url}" loading="lazy" alt="Media item ${index + 1}">`;

        const dimText = (item.width && item.height) ? `${item.width} x ${item.height}` : extension.toUpperCase();

        card.innerHTML = `
            <div class="media-card-header">
                <label class="card-select-label">
                    <input type="checkbox" class="media-checkbox" data-index="${index}" checked>
                    <span>Item ${index + 1} of ${total}</span>
                </label>
                <span class="type-tag ${item.media_type}">${item.media_type}</span>
            </div>
            <div class="media-preview-container">
                ${previewElement}
            </div>
            <div class="media-card-footer">
                <span class="resolution-info">${dimText}</span>
                <button class="primary-btn download-single-btn" data-index="${index}" type="button">Download</button>
            </div>
        `;

        const downloadBtn = card.querySelector(".download-single-btn");
        downloadBtn.addEventListener("click", () => {
            downloadItems([{ url: item.url, filename }]);
        });

        mediaGrid.appendChild(card);
    });
}

selectAllBtn.addEventListener("click", () => {
    document.querySelectorAll(".media-checkbox").forEach((cb) => {
        cb.checked = true;
    });
});

deselectAllBtn.addEventListener("click", () => {
    document.querySelectorAll(".media-checkbox").forEach((cb) => {
        cb.checked = false;
    });
});

downloadSelectedBtn.addEventListener("click", () => {
    if (!currentPost) return;

    const selectedCheckboxes = document.querySelectorAll(".media-checkbox:checked");
    if (selectedCheckboxes.length === 0) {
        showStatus("Please select at least one media item to download", "error");
        return;
    }

    const itemsToDownload = [];
    selectedCheckboxes.forEach((cb) => {
        const index = parseInt(cb.dataset.index, 10);
        const item = currentPost.items[index];
        const extension = item.media_type === "video" ? "mp4" : "jpg";
        const filename = currentPost.items.length <= 1
            ? `insta_${currentPost.shortcode}.${extension}`
            : `insta_${currentPost.shortcode}_${String(index + 1).padStart(2, "0")}.${extension}`;

        itemsToDownload.push({ url: item.url, filename });
    });

    downloadItems(itemsToDownload);
});

downloadAllBtn.addEventListener("click", () => {
    if (!currentPost) return;

    const itemsToDownload = currentPost.items.map((item, index) => {
        const extension = item.media_type === "video" ? "mp4" : "jpg";
        const filename = currentPost.items.length <= 1
            ? `insta_${currentPost.shortcode}.${extension}`
            : `insta_${currentPost.shortcode}_${String(index + 1).padStart(2, "0")}.${extension}`;

        return { url: item.url, filename };
    });

    downloadItems(itemsToDownload);
});

async function downloadItems(items) {
    if (!items || items.length === 0) return;

    showStatus(`Downloading ${items.length} item(s) to Downloads folder...`, "normal");

    try {
        const response = await fetch("/api/download", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ items }),
        });

        const result = await response.json();
        if (!result.success || !result.data) {
            throw new Error(result.error || "Download process failed on server");
        }

        const successes = result.data.filter((r) => r.success).length;
        const failures = result.data.length - successes;

        if (failures === 0) {
            showStatus(`Saved ${successes} file(s) directly to your Downloads folder`, "success");
        } else {
            showStatus(`Completed with ${successes} saved and ${failures} failed`, "error");
        }
    } catch (err) {
        showStatus(`Download error: ${err.message}`, "error");
    }
}
