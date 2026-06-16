const API = location.origin.startsWith("http") ? location.origin : "http://localhost:3000";

const params = new URLSearchParams(location.search);
const lobbyId = params.get("lobby");
const myName = params.get("player");

if (!lobbyId || !myName) {
	location.replace("index.html");
}

const hudLobby = document.querySelector("#hud_lobby");
const hudRound = document.querySelector("#hud_round");
const hudMaxRounds = document.querySelector("#hud_max_rounds");
const hudPhase = document.querySelector("#hud_phase");
const hudTrumpf = document.querySelector("#hud_trumpf");
const hudCurrent = document.querySelector("#hud_current");
const hudChooserWrap = document.querySelector("#hud_chooser_wrap");
const hudChooser = document.querySelector("#hud_chooser");
const statusOut = document.querySelector("#status_message");

const logList = document.querySelector("#log_list");
const seatList = document.querySelector("#seats");
const trickPile = document.querySelector("#trick_pile");
const trumpfCardEl = document.querySelector("#trumpf_card");
const scoreBody = document.querySelector("#scoreboard tbody");
const chooseTrumpfBox = document.querySelector("#choose_trumpf_box");
const predictBox = document.querySelector("#predict_box");
const predictMax = document.querySelector("#predict_max");
const predictKeypad = document.querySelector("#predict_keypad");
const yourHand = document.querySelector("#your_hand");
const gameOverBox = document.querySelector("#game_over_box");
const gameOverWinner = document.querySelector("#game_over_winner");

hudLobby.textContent = lobbyId;

let lastState = null;
let renderedLogCount = 0;
let socket = null;

for (const btn of chooseTrumpfBox.querySelectorAll("button[data-suit]")) {
	btn.addEventListener("click", () => onChooseTrumpf(btn.dataset.suit));
}

openSocket();

function openSocket() {
	const wsUrl =
		API.replace(/^http/, "ws") +
		`/lobby/${encodeURIComponent(lobbyId)}/ws?player=${encodeURIComponent(myName)}`;
	socket = new WebSocket(wsUrl);
	socket.addEventListener("open", () => setStatus("connected"));
	socket.addEventListener("message", ({ data }) => {
		try {
			const msg = JSON.parse(data);
			if (msg.type === "state") renderGame(msg.state);
		} catch (e) {
			console.warn("bad ws frame:", data, e);
		}
	});
	socket.addEventListener("close", () => {
		setStatus("disconnected", true);
		setTimeout(openSocket, 1500);
	});
}

async function onChooseTrumpf(suit) {
	const next = await api("POST", `/lobby/${encodeURIComponent(lobbyId)}/choose_trumpf`, {
		player: myName,
		suit,
	});
	if (next) renderGame(next);
}

async function onPredict(value) {
	if (!lastState) return;
	for (const btn of predictKeypad.querySelectorAll("button")) btn.disabled = true;
	const next = await api("POST", `/lobby/${encodeURIComponent(lobbyId)}/predict`, {
		player: myName,
		prediction: value,
	});
	if (next) renderGame(next);
}

async function onPlayCard(card_index) {
	const next = await api("POST", `/lobby/${encodeURIComponent(lobbyId)}/play`, {
		player: myName,
		card_index,
	});
	if (next) renderGame(next);
}

function suitClass(suit) {
	return ["red", "yellow", "green", "blue"].includes(suit) ? suit : "neutral";
}

function makeCardNode(card, opts = {}) {
	const node = document.createElement("div");
	node.classList.add("card");
	if (card.value === 14) {
		node.classList.add("wizard");
		node.textContent = "W";
		node.title = `Wizard (${card.suit})`;
	} else if (card.value === 0) {
		node.classList.add("jester");
		node.textContent = "J";
		node.title = `Jester (${card.suit})`;
	} else {
		node.classList.add(suitClass(card.suit));
		node.textContent = card.value;
		node.title = `${card.suit} ${card.value}`;
	}
	if (opts.dim) node.classList.add("dim");
	if (opts.highlight) node.classList.add("highlight");
	return node;
}

function playerLabel(p) {
	return p.is_ai ? `${p.name} [AI]` : p.name;
}

function appendLogLines(state) {
	const fresh = state.log.slice(renderedLogCount);
	for (const event of fresh) {
		const li = document.createElement("li");
		li.className = `log-${event.type}`;
		li.textContent = formatEvent(event, state);
		logList.appendChild(li);
	}
	renderedLogCount = state.log.length;
	logList.scrollTop = logList.scrollHeight;
}

function formatEvent(e, state) {
	const name = (i) => state.players[i] ? playerLabel(state.players[i]) : `#${i}`;
	const cardStr = (c) => {
		if (c.value === 14) return `Wizard (${c.suit})`;
		if (c.value === 0) return `Jester (${c.suit})`;
		return `${c.suit} ${c.value}`;
	};
	switch (e.type) {
		case "round_started":
			return `— round ${e.round} — dealer: ${name(e.dealer)}, flip: ${e.flipped_card ? cardStr(e.flipped_card) : "(none — no trumpf)"}`;
		case "trumpf_chosen":
			return `${name(e.chooser)} chose ${e.suit} as trumpf`;
		case "predicted":
			return `${name(e.player)} predicts ${e.prediction}`;
		case "card_played":
			return `${name(e.player)} plays ${cardStr(e.card)}`;
		case "trick_won":
			return `→ ${name(e.winner)} wins the trick`;
		case "round_scored":
			return `round ${e.round} scored: ${e.deltas.map((d, i) => `${name(i)} ${d >= 0 ? "+" : ""}${d}`).join(", ")}`;
		case "game_over":
			return `=== GAME OVER ===`;
		default:
			return JSON.stringify(e);
	}
}

function renderGame(state) {
	const firstRender = lastState === null;
	lastState = state;

	hudRound.textContent = state.round;
	hudMaxRounds.textContent = state.max_rounds;
	hudPhase.textContent = state.phase;
	hudTrumpf.textContent = state.trumpf
		? state.trumpf.value === 14
			? `wizard·${state.trumpf.suit}`
			: `${state.trumpf.suit} ${state.trumpf.value}`
		: "(none)";
	hudCurrent.textContent = playerLabel(state.players[state.current_player_index] ?? { name: "?", is_ai: false });
	if (state.trumpf_chooser_index != null) {
		hudChooserWrap.hidden = false;
		hudChooser.textContent = playerLabel(state.players[state.trumpf_chooser_index]);
	} else {
		hudChooserWrap.hidden = true;
	}

	// trumpf card on table
	trumpfCardEl.replaceChildren();
	if (state.trumpf) trumpfCardEl.appendChild(makeCardNode(state.trumpf, { highlight: true }));

	// seats around the table
	seatList.replaceChildren(
		...state.players.map((p, i) => {
			const li = document.createElement("li");
			li.className = "seat";
			if (i === state.current_player_index) li.classList.add("active");
			if (i === state.your_index) li.classList.add("you");
			if (state.trick_starter_index === i) li.classList.add("starter");
			const name = document.createElement("div");
			name.className = "seat-name";
			name.textContent = playerLabel(p) + (i === state.your_index ? " (you)" : "");
			const meta = document.createElement("div");
			meta.className = "seat-meta";
			const pred = state.predictions[i] == null ? "—" : state.predictions[i];
			meta.textContent = `pred ${pred} · won ${state.tricks_won[i]}`;
			li.append(name, meta);
			return li;
		}),
	);

	// current trick stacked center
	trickPile.replaceChildren(
		...state.current_trick.map((play) => {
			const li = document.createElement("li");
			li.className = "trick-card";
			const who = state.players[play.player_index];
			li.appendChild(makeCardNode(play.card));
			const label = document.createElement("div");
			label.className = "trick-card-label";
			label.textContent = who ? playerLabel(who) : `#${play.player_index}`;
			li.appendChild(label);
			return li;
		}),
	);

	// scoreboard
	scoreBody.replaceChildren(
		...state.players.map((p, i) => {
			const tr = document.createElement("tr");
			if (i === state.your_index) tr.classList.add("you-row");
			if (i === state.current_player_index) tr.classList.add("active-row");
			const nameTd = document.createElement("td");
			nameTd.textContent = playerLabel(p) + (i === state.your_index ? " *" : "");
			const predTd = document.createElement("td");
			predTd.textContent = state.predictions[i] == null ? "—" : state.predictions[i];
			const tricksTd = document.createElement("td");
			tricksTd.textContent = state.tricks_won[i];
			const pointsTd = document.createElement("td");
			pointsTd.textContent = state.points[i];
			tr.append(nameTd, predTd, tricksTd, pointsTd);
			return tr;
		}),
	);

	// log (append-only)
	if (firstRender) {
		// initial render: blow away log and re-render in full
		logList.replaceChildren();
		renderedLogCount = 0;
	}
	appendLogLines(state);

	// turn-specific controls
	const myTurn = state.current_player_index === state.your_index;
	chooseTrumpfBox.hidden =
		!(state.phase === "choose_trumpf" && state.trumpf_chooser_index === state.your_index);

	const isMyPredictTurn = state.phase === "prediction" && myTurn;
	predictBox.hidden = !isMyPredictTurn;
	predictMax.textContent = state.round;
	if (isMyPredictTurn) {
		predictKeypad.replaceChildren(
			...Array.from({ length: state.round + 1 }, (_, n) => {
				const btn = document.createElement("button");
				btn.type = "button";
				btn.className = "predict-key";
				btn.textContent = n;
				btn.addEventListener("click", () => onPredict(n));
				return btn;
			}),
		);
	} else {
		predictKeypad.replaceChildren();
	}

	// hand
	const isMyPlayTurn = state.phase === "playing" && myTurn;
	yourHand.replaceChildren(
		...state.your_hand.map((c, i) => {
			const legal = state.your_legal_card_indices.includes(i);
			const node = makeCardNode(c, { dim: isMyPlayTurn && !legal });
			if (isMyPlayTurn && legal) {
				node.classList.add("playable");
				node.addEventListener("click", () => onPlayCard(i));
			}
			return node;
		}),
	);

	if (state.phase === "game_over") {
		const sorted = state.players
			.map((p, i) => ({ p, pts: state.points[i] }))
			.sort((a, b) => b.pts - a.pts);
		const top = sorted[0];
		gameOverBox.hidden = false;
		gameOverWinner.textContent = `Winner: ${playerLabel(top.p)} with ${top.pts} pts.`;
	}
}

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
