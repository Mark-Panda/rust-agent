// 游戏配置
const CONFIG = {
    CANVAS_SIZE: 400,
    GRID_SIZE: 20,
    GAME_SPEED: 100,
    CELL_SIZE: 20
};

// 游戏状态
let gameState = {
    snake: [{x: 10, y: 10}],
    food: {x: 15, y: 15},
    direction: {x: 0, y: 0},
    score: 0,
    highScore: localStorage.getItem('snakeHighScore') || 0,
    gameRunning: false,
    gamePaused: false,
    gameLoop: null
};

// DOM元素
const canvas = document.getElementById('game-canvas');
const ctx = canvas.getContext('2d');
const scoreElement = document.getElementById('score');
const highScoreElement = document.getElementById('high-score');
const startBtn = document.getElementById('start-btn');
const pauseBtn = document.getElementById('pause-btn');
const restartBtn = document.getElementById('restart-btn');

// 初始化游戏
function initGame() {
    highScoreElement.textContent = gameState.highScore;
    drawGame();
    bindEvents();
}

// 绑定事件
function bindEvents() {
    startBtn.addEventListener('click', startGame);
    pauseBtn.addEventListener('click', togglePause);
    restartBtn.addEventListener('click', restartGame);
    
    document.addEventListener('keydown', handleKeyPress);
}

// 处理键盘输入
function handleKeyPress(e) {
    if (!gameState.gameRunning || gameState.gamePaused) return;
    
    const key = e.key.toLowerCase();
    let newDirection = {x: 0, y: 0};
    
    switch(key) {
        case 'arrowup':
        case 'w':
            if (gameState.direction.y === 1) return; // 防止反向移动
            newDirection = {x: 0, y: -1};
            break;
        case 'arrowdown':
        case 's':
            if (gameState.direction.y === -1) return;
            newDirection = {x: 0, y: 1};
            break;
        case 'arrowleft':
        case 'a':
            if (gameState.direction.x === 1) return;
            newDirection = {x: -1, y: 0};
            break;
        case 'arrowright':
        case 'd':
            if (gameState.direction.x === -1) return;
            newDirection = {x: 1, y: 0};
            break;
    }
    
    if (newDirection.x !== 0 || newDirection.y !== 0) {
        gameState.direction = newDirection;
    }
}

// 开始游戏
function startGame() {
    if (gameState.gameRunning) return;
    
    gameState.gameRunning = true;
    gameState.gamePaused = false;
    gameState.direction = {x: 1, y: 0};
    
    startBtn.disabled = true;
    pauseBtn.disabled = false;
    
    gameState.gameLoop = setInterval(gameStep, CONFIG.GAME_SPEED);
}

// 暂停/继续游戏
function togglePause() {
    if (!gameState.gameRunning) return;
    
    gameState.gamePaused = !gameState.gamePaused;
    
    if (gameState.gamePaused) {
        clearInterval(gameState.gameLoop);
        pauseBtn.textContent = '继续';
    } else {
        gameState.gameLoop = setInterval(gameStep, CONFIG.GAME_SPEED);
        pauseBtn.textContent = '暂停';
    }
}

// 重新开始游戏
function restartGame() {
    clearInterval(gameState.gameLoop);
    
    gameState = {
        snake: [{x: 10, y: 10}],
        food: generateFood(),
        direction: {x: 0, y: 0},
        score: 0,
        highScore: gameState.highScore,
        gameRunning: false,
        gamePaused: false,
        gameLoop: null
    };
    
    startBtn.disabled = false;
    pauseBtn.disabled = true;
    pauseBtn.textContent = '暂停';
    
    scoreElement.textContent = '0';
    drawGame();
}

// 游戏步进
function gameStep() {
    moveSnake();
    
    if (checkCollision()) {
        gameOver();
        return;
    }
    
    if (checkFoodCollision()) {
        eatFood();
    }
    
    drawGame();
}

// 移动蛇
function moveSnake() {
    const head = {...gameState.snake[0]};
    head.x += gameState.direction.x;
    head.y += gameState.direction.y;
    
    gameState.snake.unshift(head);
    
    // 如果没有吃到食物，移除尾部
    if (!checkFoodCollision()) {
        gameState.snake.pop();
    }
}

// 检查碰撞
function checkCollision() {
    const head = gameState.snake[0];
    
    // 检查墙壁碰撞
    if (head.x < 0 || head.x >= CONFIG.GRID_SIZE || 
        head.y < 0 || head.y >= CONFIG.GRID_SIZE) {
        return true;
    }
    
    // 检查自身碰撞
    for (let i = 1; i < gameState.snake.length; i++) {
        if (head.x === gameState.snake[i].x && head.y === gameState.snake[i].y) {
            return true;
        }
    }
    
    return false;
}

// 检查食物碰撞
function checkFoodCollision() {
    const head = gameState.snake[0];
    return head.x === gameState.food.x && head.y === gameState.food.y;
}

// 吃食物
function eatFood() {
    gameState.score += 10;
    scoreElement.textContent = gameState.score;
    
    if (gameState.score > gameState.highScore) {
        gameState.highScore = gameState.score;
        highScoreElement.textContent = gameState.highScore;
        localStorage.setItem('snakeHighScore', gameState.highScore);
    }
    
    gameState.food = generateFood();
}

// 生成食物
function generateFood() {
    let newFood;
    do {
        newFood = {
            x: Math.floor(Math.random() * CONFIG.GRID_SIZE),
            y: Math.floor(Math.random() * CONFIG.GRID_SIZE)
        };
    } while (gameState.snake.some(segment => segment.x === newFood.x && segment.y === newFood.y));
    
    return newFood;
}

// 绘制游戏
function drawGame() {
    // 清空画布
    ctx.fillStyle = '#f0f0f0';
    ctx.fillRect(0, 0, CONFIG.CANVAS_SIZE, CONFIG.CANVAS_SIZE);
    
    // 绘制网格线
    ctx.strokeStyle = '#ddd';
    ctx.lineWidth = 1;
    for (let i = 0; i <= CONFIG.GRID_SIZE; i++) {
        ctx.beginPath();
        ctx.moveTo(i * CONFIG.CELL_SIZE, 0);
        ctx.lineTo(i * CONFIG.CELL_SIZE, CONFIG.CANVAS_SIZE);
        ctx.stroke();
        
        ctx.beginPath();
        ctx.moveTo(0, i * CONFIG.CELL_SIZE);
        ctx.lineTo(CONFIG.CANVAS_SIZE, i * CONFIG.CELL_SIZE);
        ctx.stroke();
    }
    
    // 绘制蛇
    gameState.snake.forEach((segment, index) => {
        if (index === 0) {
            // 蛇头
            ctx.fillStyle = '#4CAF50';
        } else {
            // 蛇身
            ctx.fillStyle = '#81C784';
        }
        
        ctx.fillRect(
            segment.x * CONFIG.CELL_SIZE + 2,
            segment.y * CONFIG.CELL_SIZE + 2,
            CONFIG.CELL_SIZE - 4,
            CONFIG.CELL_SIZE - 4
        );
    });
    
    // 绘制食物
    ctx.fillStyle = '#f44336';
    ctx.beginPath();
    ctx.arc(
        gameState.food.x * CONFIG.CELL_SIZE + CONFIG.CELL_SIZE / 2,
        gameState.food.y * CONFIG.CELL_SIZE + CONFIG.CELL_SIZE / 2,
        CONFIG.CELL_SIZE / 2 - 2,
        0,
        2 * Math.PI
    );
    ctx.fill();
}

// 游戏结束
function gameOver() {
    clearInterval(gameState.gameLoop);
    gameState.gameRunning = false;
    
    startBtn.disabled = false;
    pauseBtn.disabled = true;
    
    alert(`游戏结束！你的分数是: ${gameState.score}`);
}

// 初始化游戏
initGame();