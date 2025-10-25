<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { open, message } from "@tauri-apps/plugin-dialog";
import { LazyStore } from "@tauri-apps/plugin-store";
import { relaunch } from "@tauri-apps/plugin-process";
import { Event, listen, UnlistenFn } from "@tauri-apps/api/event";

const currentView = ref<"settings" | "logs">("settings");

const store = new LazyStore("settings.json");

const selectedPath = ref("");
const timeToBackupInMinutes = ref(0);
const maxBackups = ref(0);

const logMessages = ref<string[]>([]);
const unlisten = ref<UnlistenFn>();

async function selectPath() {
	const result = await open({ directory: true, multiple: false });
	if (result) {
		selectedPath.value = result as string;
	}
}

async function save() {
	try {
		store.set("selected_path", selectedPath.value);
		store.set("time_to_backup", timeToBackupInMinutes.value);
		store.set("max_backups", maxBackups.value);
		await store.save();

		await message("Configurações Salvas", "Success");

		await relaunch();
	} catch (error) {
		await message("Falha ao salvar local de backup: " + error, "Error");
	}
}

function normalizeText(text: string): string {
	const normalized = text.normalize("NFC");
	if (normalized.length > 50) {
		return normalized.slice(0, 47) + "...";
	}
	return normalized;
}

onMounted(async () => {
	const path = await store.get("selected_path");
	const time = await store.get("time_to_backup");
	const max = await store.get("max_backups");

	if (path && typeof path === "string") {
		selectedPath.value = path;
	}
	if (time && typeof time === "number") {
		timeToBackupInMinutes.value = time;
	}
	if (max && typeof max === "number") {
		maxBackups.value = max;
	}

	unlisten.value = await listen("log_event", (event: Event<string>) => {
		const length = logMessages.value.length;

		if (length > 50) logMessages.value.shift();

		logMessages.value.push(event.payload);
	});
});

onBeforeUnmount(() => {
	unlisten.value?.();
});
</script>

<template>
	<div class="page-bg">
		<div class="nav">
			<button :class="['choose-btn', currentView === 'settings' ? 'active' : '']" @click="currentView = 'settings'" type="button">
				Settings
			</button>
			<button :class="['choose-btn', currentView === 'logs' ? 'active' : '']" @click="currentView = 'logs'" type="button">Logs</button>
		</div>

		<div v-if="currentView === 'settings'" class="card">
			<div>
				<span class="card-title">Backup Settings</span>
				<p class="card-desc">Select a folder on your computer where backups will be saved</p>
			</div>
			<div style="width: 100%">
				<div class="field-label">Backup Folder Location</div>
				<div class="input-row">
					<button class="choose-btn" @click="selectPath" type="button">
						<svg
							width="22"
							height="22"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#222"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
						</svg>
						<p style="margin: 0px; transform: translateY(2px)">{{ normalizeText(selectedPath || "Choose folder...") }}</p>
					</button>
				</div>
			</div>

			<div style="width: 100%">
				<div class="field-label">Backup Interval (minutes)</div>
				<div class="input-row">
					<input
						type="number"
						v-model="timeToBackupInMinutes"
						min="1"
						style="
							width: 100%;
							padding: 0.5rem 0.75rem;
							font-size: 1rem;
							border-radius: 8px;
							border: 1px solid #e0e0e0;
							box-shadow: 0 1px 4px #0001;
						"
					/>
				</div>
			</div>

			<div style="width: 100%">
				<div class="field-label">Maximum Number of Backups</div>
				<div class="input-row">
					<input
						type="number"
						v-model="maxBackups"
						min="1"
						style="
							width: 100%;
							padding: 0.5rem 0.75rem;
							font-size: 1rem;
							border-radius: 8px;
							border: 1px solid #e0e0e0;
							box-shadow: 0 1px 4px #0001;
						"
					/>
				</div>
			</div>

			<button class="save-btn" @click="save" type="button">Save Configuration</button>
		</div>

		<div v-else class="card">
			<div>
				<span class="card-title">Backup Logs</span>
				<p class="card-desc">Recent backup activity logs</p>
			</div>
			<div class="logs">
				<div v-for="(log, index) in logMessages" :key="index" style="font-size: 0.9rem; color: #444; margin-bottom: 0.4rem">
					{{ log }}
				</div>
			</div>
			<button class="save-btn" @click="logMessages = []" type="button" style="margin-top: 1rem">Limpar logs</button>
		</div>
	</div>
</template>

<style>
html,
body {
	margin: 0;
	padding: 0;
	font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
	background: #fafafa;
}
</style>

<style scoped>
.page-bg {
	min-height: 100vh;
	display: flex;
	align-items: center;
	justify-content: center;
	background: linear-gradient(180deg, #fff 60%, #fafafa 100%);
}

.nav {
	position: absolute;
	top: 1rem;
	left: 50%;
	transform: translateX(-50%);
	display: flex;
	gap: 0.5rem;
}

.card {
	background: #fff;
	border-radius: 16px;
	box-shadow: 0 2px 16px #0002;
	padding: 1.25rem;
	min-width: 460px;
	max-width: 460px;
	max-height: 500px;
	display: flex;
	flex-direction: column;
	gap: 1.2rem;
	align-items: stretch;
}
.card-title {
	font-size: 1.25rem;
	font-weight: 700;
	color: #222;
	margin-bottom: 0.2rem;
	margin-top: 0px;
}
.card-desc {
	font-size: 1rem;
	color: #888;
	margin-top: 0px;
	margin-bottom: 0px;
}
.field-label {
	font-size: 1rem;
	color: #222;
	font-weight: 500;
	margin-bottom: 0.3rem;
}
.input-row {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	width: 100%;
}
.folder-icon {
	display: flex;
	align-items: center;
	justify-content: center;
	background: #f5f5f5;
	border-radius: 8px;
	padding: 0.3rem 0.5rem;
	margin-right: 0.2rem;
}
.choose-btn {
	padding: 0.5rem 1rem;
	border-radius: 8px;
	border: none;
	font-size: 1rem;
	font-weight: 500;
	background: #fff;
	color: #222;
	box-shadow: 0 1px 4px #0001;
	cursor: pointer;
	border: 1px solid #e0e0e0;
	transition: background 0.2s, border-color 0.2s;
	width: 100%;
	display: flex;
	flex-direction: row;
	align-items: center;
	gap: 1rem;
}
.choose-btn:hover {
	background: #f5f5f5;
	border-color: #222;
}
.save-btn {
	margin-top: 0.5rem;
	padding: 0.9rem 0;
	border-radius: 8px;
	border: none;
	font-size: 1.1rem;
	font-weight: 600;
	background: #222;
	color: #fff;
	box-shadow: 0 2px 8px #0002;
	cursor: pointer;
	transition: background 0.2s;
}
.save-btn:hover {
	background: #222222f0;
}
.logs {
	max-height: 400px;
	overflow-y: auto;
	margin-top: 1rem;
	border: 1px solid #e0e0e0;
	border-radius: 8px;
	padding: 0.5rem;
	background: #f9f9f9;
}
</style>
