# Yggdrasil Vision

> [English](./VISION.md) · [中文](./VISION.zh-CN.md)

Yggdrasil 是一个面向 AI 原生世界、游戏、故事和游玩的扩展驱动创作平台。

平台的核心小且内容无关。所有有意义的概念都存在于能力包中，内核以平等的方式承载它们。

## Yggdrasil 是什么

一个承载能力包的内核。

一份让客户端、能力包和外部系统以平等身份参与的公开协议。

一个保留已发生之事的事件溯源底座。

一个供激进 AI 原生体验使用的创作 surface——平台本身不预定义这些体验。

## Yggdrasil 不是什么

不是应用。不是聊天工具。不是 SillyTavern 替代品。不是内置了类型的框架。不是一个中心填满了特权官方内容的 plugin host。

平台不会发布任何典范体验。内核对角色、世界、提示词、模型、agent 或记忆没有任何立场。那些是能力包关注的事。

## 「激进创作自由」在这里意味着什么

创作者不被局限于 Yggdrasil 所想象的形态。

创作者可以：

- 定义自己的类型、循环、规则和呈现方式，
- 将 AI 行为作为构建块组合，
- 审视、fork、改写和重组任何体验，
- 用自己的能力包替换或覆盖任何官方包，
- 分发新的 capability、新的 event kind、新的 extension point，
- 在同一个 session 中混合多个能力包，没有任何一方享有特权。

平台的职责是让这一切成为可能。平台的职责不是提供体验。

## 为什么是内核加能力包

封闭框架决定了媒介是什么。Yggdrasil 拒绝这样做。

把所有含义放在能力包中——包括官方包——保持了媒介的开放性。它保持了平台的诚实：如果一个官方"对话 runtime"可以被替换，或者与一个第三方"世界模拟器"共存，那内核就不是暗中的掌控者。

这就是在时间维度上保护创作自由的方式。

## 适用范围

Yggdrasil 被设计为以下角色可用：

- 本地平台 host，
- 讲公开协议的无头服务，
- 嵌入更大产品中的库，
- 供外部系统作为能力包或客户端使用的开放协议。

四者使用同一份契约。

## 未来 capability 家族（已延后）

以下是有价值的方向，但它们是能力包，不是内核关注的事。它们要等到内核/能力包层稳定之后。

- SillyTavern 资源和行为兼容能力包家族。
- agent 集成能力包家族（pi 或其他）。
- 游戏引擎桥接能力包家族（UE5、Godot、Unity、web 客户端）。
- 官方对话 runtime 能力包。
- 官方审查器和创作者 UI。

每一个都将作为普通能力包构建和评判。没有一个会获得内核特权。

## 非目标

内核不会发布聊天体验、世界模拟器、director、记忆模型、SillyTavern 兼容层、外部引擎桥接或官方 UI。

每一个作为能力包都是合适的。没有一个作为内核是合适的。

## 对当前代码的态度

Rust workspace 目前是 Platform Foundation Alpha：仅内核的 event/session、基于 manifest 的能力包、真正的 `rust_inproc` 和 subprocess 执行、hook fabric、SQLite event 日志、权限化 principal、surface contribution、proposal/approval 生命周期、asset/branch/projection 底座、以及走公开协议的 web shell。当前的纪律是防止契约漂移——surface、proposal、branch、asset 和 projection 必须保持其通用形态，内容形态的语义不能泄漏到内核中，官方包必须只使用任何第三方包都能使用的东西。

## 成功的样子

当一个创作者可以构建出平台作者未曾预见的东西，把它作为能力包发布，并与官方包并行运行且不受二等对待时，Yggdrasil 就成功了。
