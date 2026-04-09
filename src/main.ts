import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

let selectedFilePath: string | null = null;
let calculatedMd5: string | null = null;

// DOM elements
const fileNameEl = document.querySelector("#file-name") as HTMLElement;
const selectFileBtn = document.querySelector("#select-file-btn") as HTMLButtonElement;
const calculateBtn = document.querySelector("#calculate-btn") as HTMLButtonElement;
const addMd5Btn = document.querySelector("#add-md5-btn") as HTMLButtonElement;
const md5ResultEl = document.querySelector("#md5-result") as HTMLElement;
const copyBtn = document.querySelector("#copy-btn") as HTMLButtonElement;
const statusMsgEl = document.querySelector("#status-msg") as HTMLElement;

function showStatus(message: string, isError: boolean = false) {
  statusMsgEl.textContent = message;
  statusMsgEl.className = `status-msg ${isError ? "error" : "success"}`;
  setTimeout(() => {
    statusMsgEl.textContent = "";
    statusMsgEl.className = "status-msg";
  }, 3000);
}

function enableButtons(enabled: boolean) {
  calculateBtn.disabled = !enabled;
  addMd5Btn.disabled = !enabled;
}

// 选择文件
async function selectFile() {
  try {
    const file = await open({
      multiple: false,
      filters: [
        { name: "SQL", extensions: ["sql"] },
        { name: "Groovy", extensions: ["groovy"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });

    if (file) {
      selectedFilePath = file as string;
      const fileName = selectedFilePath.split(/[/\\]/).pop() || selectedFilePath;
      fileNameEl.textContent = fileName;
      enableButtons(true);
      md5ResultEl.textContent = "-";
      calculatedMd5 = null;
      copyBtn.disabled = true;
    }
  } catch (error) {
    showStatus(`选择文件失败: ${error}`, true);
  }
}

// 计算MD5（不修改文件）
async function calculateMd5() {
  if (!selectedFilePath) {
    showStatus("请先选择文件", true);
    return;
  }

  try {
    calculateBtn.disabled = true;
    calculateBtn.textContent = "计算中...";

    const result = await invoke<string>("calculate_md5", {
      filePath: selectedFilePath,
    });

    calculatedMd5 = result;
    md5ResultEl.textContent = result;
    copyBtn.disabled = false;
    showStatus("MD5计算成功");
  } catch (error) {
    showStatus(`计算失败: ${error}`, true);
  } finally {
    calculateBtn.disabled = false;
    calculateBtn.textContent = "计算MD5";
  }
}

// 添加MD5到文件
async function addMd5ToFile() {
  if (!selectedFilePath) {
    showStatus("请先选择文件", true);
    return;
  }

  try {
    addMd5Btn.disabled = true;
    addMd5Btn.textContent = "处理中...";

    const result = await invoke<string>("add_md5_to_file", {
      filePath: selectedFilePath,
    });

    calculatedMd5 = result;
    md5ResultEl.textContent = result;
    copyBtn.disabled = false;
    showStatus("MD5已成功写入文件");
  } catch (error) {
    showStatus(`写入失败: ${error}`, true);
  } finally {
    addMd5Btn.disabled = false;
    addMd5Btn.textContent = "添加MD5到文件";
  }
}

// 复制MD5
async function copyMd5() {
  if (!calculatedMd5) {
    return;
  }

  try {
    await navigator.clipboard.writeText(calculatedMd5);
    showStatus("MD5已复制到剪贴板");
  } catch (error) {
    showStatus(`复制失败: ${error}`, true);
  }
}

// 初始化事件监听
window.addEventListener("DOMContentLoaded", () => {
  selectFileBtn.addEventListener("click", selectFile);
  calculateBtn.addEventListener("click", calculateMd5);
  addMd5Btn.addEventListener("click", addMd5ToFile);
  copyBtn.addEventListener("click", copyMd5);
});