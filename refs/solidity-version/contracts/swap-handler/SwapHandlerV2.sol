// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.28;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

interface IUniswapV2Router02 {
    function WETH() external pure returns (address);
    function swapExactTokensForTokensSupportingFeeOnTransferTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external;
    function swapExactETHForTokensSupportingFeeOnTransferTokens(
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external payable;
    function swapExactTokensForETHSupportingFeeOnTransferTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external;
}

contract SwapHandlerV2 is Ownable {
    using SafeERC20 for IERC20;

    IUniswapV2Router02 public ROUTER;
    address public WETH;

    event RouterUpdated(IUniswapV2Router02 indexed oldRouter, IUniswapV2Router02 indexed newRouter);
    event TokenSwapped(address indexed sender, address indexed tokenIn, address indexed tokenOut, string comment);

    constructor(address _owner, IUniswapV2Router02 _router) {
        _transferOwnership(_owner);
        _updateRouter(_router);
    }

    function wrapSwapExactTokensForTokens(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline,
        string calldata comment
    ) external {
        require(path.length >= 2, "Invalid path");
        IERC20(path[0]).safeTransferFrom(msg.sender, address(this), amountIn);
        IERC20(path[0]).safeApprove(address(ROUTER), amountIn);
        ROUTER.swapExactTokensForTokensSupportingFeeOnTransferTokens(amountIn, amountOutMin, path, to, deadline);
        emit TokenSwapped(msg.sender, path[0], path[path.length - 1], comment);
    }

    function wrapSwapExactETHForTokens(
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline,
        string calldata comment
    ) external payable {
        require(path.length >= 2, "Invalid path");
        require(path[0] == WETH, "Path must start with WETH");
        ROUTER.swapExactETHForTokensSupportingFeeOnTransferTokens{ value: msg.value }(amountOutMin, path, to, deadline);
        emit TokenSwapped(msg.sender, path[0], path[path.length - 1], comment);
    }

    function wrapSwapExactTokensForETH(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline,
        string calldata comment
    ) external {
        require(path.length >= 2, "Invalid path");
        require(path[path.length - 1] == WETH, "Path must end with WETH");
        IERC20(path[0]).safeTransferFrom(msg.sender, address(this), amountIn);
        IERC20(path[0]).safeApprove(address(ROUTER), amountIn);
        ROUTER.swapExactTokensForETHSupportingFeeOnTransferTokens(amountIn, amountOutMin, path, to, deadline);
        emit TokenSwapped(msg.sender, path[0], path[path.length - 1], comment);
    }

    function _updateRouter(IUniswapV2Router02 _newRouter) internal {
        IUniswapV2Router02 _oldRouter = ROUTER;
        ROUTER = _newRouter;
        WETH = _newRouter.WETH();
        emit RouterUpdated(_oldRouter, _newRouter);
    }

    function setRouter(IUniswapV2Router02 _newRouter) external onlyOwner {
        _updateRouter(_newRouter);
    }

    function emergencyWithdraw(address token, address to, uint256 value) external onlyOwner {
        if (token == address(0)) {
            payable(to).transfer(value);
        } else {
            IERC20(token).safeTransfer(to, value);
        }
    }

    receive() external payable {}
}
