const canvas = document.getElementById('game-canvas');
const ctx = canvas.getContext('2d');

// 设置画布大小
canvas.width = 400;
canvas.height = 400;

// 游戏参数
const gridSize = 20;
const tileCount = canvas.width / gridSize;
let score = 0;

// 蛇的初始状态
let snake = [
    {x: 10, y: 10}
];
let velocityX = 0;
let velocityY = 0;

// 食物位置
let foodX = 5;
let foodY = 5;

// 游戏循环
function gameLoop() {
    // 移动蛇
    moveSnake();
    
    // 检查碰撞
    if (isGameOver()) {
        alert('游戏结束! 得分: ' + score);
        resetGame();
        return;
    }
    
    // 清空画布
    ctx.fillStyle = 'white';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    
    // 绘制食物
    drawFood();
    
    // 绘制蛇
    drawSnake();
    
    // 显示分数
    drawScore();
    
    // 继续游戏循环
    setTimeout(gameLoop, 100);
}

// 移动蛇
function moveSnake() {
    const head = {x: snake[0].x + velocityX, y: snake[0].y + velocityY};
    
    // 检查是否吃到食物
    if (head.x === foodX && head.y === foodY) {
        // 增加分数
        score += 10;
        
        // 生成新食物
        generateFood();
    } else {
        // 移除蛇尾
        snake.pop();
    }
    
    // 添加新头部
    snake.unshift(head);
}

// 绘制蛇
function drawSnake() {
    ctx.fillStyle = 'green';
    snake.forEach(part => {
        ctx.fillRect(part.x * gridSize, part.y * gridSize, gridSize, gridSize);
    });
}

// 绘制食物
function drawFood() {
    ctx.fillStyle = 'red';
    ctx.fillRect(foodX * gridSize, foodY * gridSize, gridSize, gridSize);
}

// 生成食物
function generateFood() {
    foodX = Math.floor(Math.random() * tileCount);
    foodY = Math.floor(Math.random() * tileCount);
    
    // 确保食物不会出现在蛇身上
    snake.forEach(part => {
        if (part.x === foodX && part.y === foodY) {
            generateFood();
        }
    });
}

// 检查游戏结束条件
function isGameOver() {
    const head = snake[0];
    
    // 撞墙
    if (head.x < 0 || head.x >= tileCount || head.y < 0 || head.y >= tileCount) {
        return true;
    }
    
    // 撞到自己
    for (let i = 1; i < snake.length; i++) {
        if (head.x === snake[i].x && head.y === snake[i].y) {
            return true;
        }
    }
    
    return false;
}

// 重置游戏
function resetGame() {
    snake = [{x: 10, y: 10}];
    velocityX = 0;
    velocityY = 0;
    score = 0;
    generateFood();
}

// 绘制分数
function drawScore() {
    ctx.fillStyle = 'black';
    ctx.font = '20px Arial';
    ctx.fillText('得分: ' + score, 10, 20);
}

// 键盘控制
document.addEventListener('keydown', (e) => {
    switch (e.key) {
        case 'ArrowUp':
            if (velocityY !== 1) {
                velocityX = 0;
                velocityY = -1;
            }
            break;
        case 'ArrowDown':
            if (velocityY !== -1) {
                velocityX = 0;
                velocityY = 1;
            }
            break;
        case 'ArrowLeft':
            if (velocityX !== 1) {
                velocityX = -1;
                velocityY = 0;
            }
            break;
        case 'ArrowRight':
            if (velocityX !== -1) {
                velocityX = 1;
                velocityY = 0;
            }
            break;
    }
});

// 初始化游戏
generateFood();
gameLoop();