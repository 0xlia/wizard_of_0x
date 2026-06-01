const API = location.origin.startsWith("http") ? location.origin : "http://localhost:3000";

const createForm = document.querySelector("#create_game_input");
const joinForm = document.querySelector("#join_game_input");
const gameIdOut = document.querySelector("#current_gameid");
const playersOut = document.querySelector("#current_players");
const statusOut = document.querySelector("#status_message");
const startBtn = document.querySelector("#start_game_btn");
const addBotBtn = document.querySelector("#add_bot_btn");

let currentLobby = null;
let myName = null;
let pollTimer = null;

createForm.addEventListener("submit", onCreate);
joinForm.addEventListener("submit", onJoin);
startBtn.addEventListener("click", onStart);
addBotBtn.addEventListener("click", onAddBot);

async function onCreate(evt) {
	evt.preventDefault();
	const playername = document.querySelector("#playername_create").value.trim();
	const num_players = Number(document.querySelector("#playernumber").value);

	if (!playername) return setStatus("Pick a name first.", true);
	if (!(num_players >= 3 && num_players <= 6))
		return setStatus("Player count must be 3-6.", true);

	const lobby = await api("POST", "/lobby", { playername, num_players });
	if (lobby) {
		myName = playername;
		enterLobby(lobby);
	}
}

async function onJoin(evt) {
	evt.preventDefault();
	const playername = document.querySelector("#playername_join").value.trim();
	const id = document.querySelector("#gameidinput").value.trim();

	if (!playername) return setStatus("Pick a name first.", true);
	if (!id) return setStatus("Need a game ID.", true);

	const lobby = await api("POST", `/lobby/${encodeURIComponent(id)}/join`, {
		playername,
	});
	if (lobby) {
		myName = playername;
		enterLobby(lobby);
	}
}

function enterLobby(lobby) {
	currentLobby = lobby.id;
	renderLobby(lobby);
	setStatus(`In lobby ${lobby.id}.`);
	startPolling();
}

function renderLobby(lobby) {
	gameIdOut.textContent = lobby.id;
	const names = lobby.players.map((p) => (p.is_bot ? `${p.name} [BOT]` : p.name));
	playersOut.textContent = `${names.join(", ")} (${lobby.players.length}/${lobby.num_players})`;
	const full = lobby.players.length >= lobby.num_players;
	startBtn.hidden = lobby.started || !full;
	addBotBtn.hidden = lobby.started || full;
	if (lobby.started) {
		stopPolling();
		goToGame();
	}
}

function goToGame() {
	const url = `game.html?lobby=${encodeURIComponent(currentLobby)}&player=${encodeURIComponent(myName)}`;
	location.assign(url);
}

async function onStart() {
	if (!currentLobby) return;
	startBtn.disabled = true;
	const lobby = await api("POST", `/lobby/${encodeURIComponent(currentLobby)}/start`);
	startBtn.disabled = false;
	if (lobby) renderLobby(lobby);
}

async function onAddBot() {
	if (!currentLobby) return;
	addBotBtn.disabled = true;
	const lobby = await api("POST", `/lobby/${encodeURIComponent(currentLobby)}/add_bot`);
	addBotBtn.disabled = false;
	if (lobby) renderLobby(lobby);
}


// refresh lobby every 2 seconds
function startPolling() {
	stopPolling();
	pollTimer = setInterval(refreshLobby, 2000);
}

async function refreshLobby() {
	if (!currentLobby) return;
	const lobby = await api("GET", `/lobby/${encodeURIComponent(currentLobby)}`);
	if (lobby) renderLobby(lobby);
}

// stop refresh 
function stopPolling() {
	if (pollTimer) {
		clearInterval(pollTimer);
		pollTimer = null;
	}
}

// api calls
async function api(method, path, body) {
	try {
		const res = await fetch(API + path, {
			method,
			headers: body ? { "Content-Type": "application/json" } : undefined,
			body: body ? JSON.stringify(body) : undefined,
		});
		const text = await res.text();
		const json = text ? JSON.parse(text) : null;
		if (!res.ok) {
			setStatus(json?.error ?? `HTTP ${res.status}`, true);
			return null;
		}
		return json;
	} catch (e) {
		setStatus(`Network error: ${e.message}`, true);
		return null;
	}
}

function setStatus(msg, isError = false) {
	if (!statusOut) return;
	statusOut.textContent = msg;
	statusOut.style.color = isError ? "var(--orange-text)" : "var(--green-text)";
}
