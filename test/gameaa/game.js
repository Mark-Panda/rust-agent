// 游戏配置
const config = {
    gravity: 0.5,
    jumpPower: 0,
    maxJumpPower: 20,
    platformSpeed: 2,
    platformGap: 200,
    platformWidth: 100,
    platformHeight: 20
};

// 游戏状态
let gameState = {
    score: 0,
    isGameOver: false,
    isCharging: false,
    chargeStartTime: 0
};

// 游戏对象
const canvas = document.getElementById('gameCanvas');
const ctx = canvas.getContext('2d');
const scoreElement = document.getElementById('score');
const gameOverElement = document.getElementById('gameOver');
const finalScoreElement = document.getElementById('finalScore');
const restartBtn = document.getElementById('restartBtn');

// 玩家对象
const player = {
    x: 100,
    y: 400,
    width: 30,
    height: 30,
    velocityX: 0,
    velocityY: 0,
    isJumping: false,
    color: '#FF6B6B'
};

// 平台数组
let platforms = [];

// 初始化平台
function initPlatforms() {
    platforms = [
        { x: 50, y: 450, width: 200, height: 20, color: '#4ECDC4' },
        { x: 300, y: 400, width: 100, height: 20, color: '#45B7D1' },
        { x: 500, y: 350, width: 120, height: 20, color: '#96CEB4' },
        { x: 700, y: 300, width: 100, height: 20, color: '#FECA57' }
    ];
}

// 生成新平台
function generatePlatform() {
    const lastPlatform = platforms[platforms.length - 1];
    const newX = lastPlatform.x + config.platformGap + Math.random() * 100;
    const newY = Math.max(200, Math.min(450, lastPlatform.y + (Math.random() - 0.5) * 100));
    const newWidth = config.platformWidth + Math.random() * 50;
    
    platforms.push({
        x: newX,
        y: newY,
        width: newWidth,
        height: config.platformHeight,
        color: `hsl(${Math.random() * 360}, 70%, 60%)`
    });
}

// 绘制玩家
function drawPlayer() {
    ctx.fillStyle = player.color;
    ctx.fillRect(player.x, player.y, player.width, player.height);
    
    // 绘制蓄力指示器
    if (gameState.isCharging) {
        const chargeTime = Date.now() - gameState.chargeStartTime;
        const chargeRatio = Math.min(chargeTime / 1000, 1);
        const power = chargeRatio * config.maxJumpPower;
        
        ctx.fillStyle = 'rgba(255, 0, 0, 0.5)';
        ctx.fillRect(player.x - 5, player.y - 20, 40 * chargeRatio, 10);
        
        ctx.fillStyle = 'white';
        ctx.font = '12px Arial';
        ctx.fillText(Math.round(power), player.x + 5, player.y - 10);
    }
}

// 绘制平台
function drawPlatforms() {
    platforms.forEach(platform => {
        ctx.fillStyle = platform.color;
        ctx.fillRect(platform.x, platform.y, platform.width, platform.height);
        
        // 绘制平台边框
        ctx.strokeStyle = '#333';
        ctx.lineWidth = 2;
        ctx.strokeRect(platform.x, platform.y, platform.width, platform.height);
    });
}

// 更新游戏状态
function update() {
    if (gameState.isGameOver) return;

    // 更新玩家位置
    player.velocityY += config.gravity;
    player.x += player.velocityX;
    player.y += player.velocityY;

    // 检测与平台的碰撞
    let onPlatform = false;
    platforms.forEach(platform => {
        if (player.x < platform.x + platform.width &&
            player.x + player.width > platform.x &&
            player.y + player.height > platform.y &&
            player.y + player.height < platform.y + platform.height + 10 &&
            player.velocityY > 0) {
            
            player.y = platform.y - player.height;
            player.velocityY = 0;
            player.velocityX = 0;
            player.isJumping = false;
            onPlatform = true;
            
            // 更新分数
            if (platform.x > 50) {
                gameState.score = Math.max(gameState.score, Math.floor(platform.x / 100));
                scoreElement.textContent = `得分: ${gameState.score}`;
            }
        }
    });

    // 移动平台
    if (player.x > canvas.width / 2) {
        const offset = player.x - canvas.width / 2;
        player.x = canvas.width / 2;
        platforms.forEach(platform => {
            platform.x -= offset;
        });
    }

    // 生成新平台
    if (platforms[platforms.length - 1].x < canvas.width + 200) {
        generatePlatform();
    }

    // 检查游戏结束条件
    if (player.y > canvas.height) {
        gameOver();
    }

    // 清理远离屏幕的平台
    platforms = platforms.filter(platform => platform.x + platform.width > -200);
}

// 绘制游戏
function draw() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // 绘制背景
    const gradient = ctx.createLinearGradient(0, 0, 0, canvas.height);
    gradient.addColorStop(0, '#87CEEB');
    gradient.addColorStop(1, '#E0F6FF');
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    
    drawPlatforms();
    drawPlayer();
}

// 游戏循环
function gameLoop() {
    update();
    draw();
    requestAnimationFrame(gameLoop);
}

// 游戏结束
function gameOver() {
    gameState.isGameOver = true;
    finalScoreElement.textContent = `最终得分: ${gameState.score}`;
    gameOverElement.style.display = 'block';
}

// 重新开始游戏
function restartGame() {
    gameState = {
        score: 0,
        isGameOver: false,
        isCharging: false,
        chargeStartTime: 0
    };
    
    player.x = 100;
    player.y = 400;
    player.velocityX = 0;
    player.velocityY = 0;
    player.isJumping = false;
    
    initPlatforms();
    scoreElement.textContent = `得分: 0`;
    gameOverElement.style.display = 'none';
}

// 事件监听
document.addEventListener('keydown', (e) => {
    if (e.code === 'Space' && !player.isJumping && !gameState.isGameOver) {
        e.preventDefault();
        if (!gameState.isCharging) {
            gameState.isCharging = true;
            gameState.chargeStartTime = Date.now();
        }
    }
});

document.addEventListener('keyup', (e) => {
    if (e.code === 'Space' && gameState.isCharging && !gameState.isGameOver) {
        e.preventDefault();
        gameState.isCharging = false;
        
        const chargeTime = Date.now() - gameState.chargeStartTime;
        const chargeRatio = Math.min(chargeTime / 1000, 1);
        const power = chargeRatio * config.maxJumpPower;
        
        player.velocityX = power * 2;
        player.velocityY = -power * 1.5;
        player.isJumping = true;
    }
});

restartBtn.addEventListener('click', restartGame);

// 初始化游戏
initPlatforms();
gameLoop();