# HTMLトークナイザー

## HTMLパーサーの流れ

<https://html.spec.whatwg.org/multipage/parsing.html#tree-construction>

### Insertion Mode

一部のみ。

- Initial → BeforeHTML: 文書の開始処理
- BeforeHead → InHead: head要素の処理開始
- InHead → AfterHead: メタデータ処理の完了
- AfterHead → InBody: メインコンテンツの処理開始
- InBody → AfterBody: 本文処理の完了
- AfterAfterBody → [*]: 文書処理の完了

### エッジケース

- 予期しないコンテンツによる状態の巻き戻し（例：AfterAfterBody → InBody）
- 必須要素の自動生成
- クリーンアップ処理

## ステートマシン図

```mermaid
stateDiagram-v2
    [*] --> Initial: Document Start
    Initial --> BeforeHtml: Initial Token

    BeforeHtml --> BeforeHead: html tag
    BeforeHtml --> BeforeHead: other token
    note right of BeforeHtml
        Creates html element
        if not exists
    end note

    BeforeHead --> InHead: head tag
    BeforeHead --> InHead: other token
    note right of BeforeHead
        Creates head element
        if not exists
    end note

    InHead --> AfterHead: head closing tag
    InHead --> AfterHead: non-head element
    note right of InHead
        Processes metadata,
        title, scripts, etc.
    end note

    AfterHead --> InBody: body tag
    AfterHead --> InBody: other token
    note right of AfterHead
        Creates body element
        if not exists
    end note

    InBody --> AfterBody: body closing tag
    note right of InBody
        Main content processing
        - Elements
        - Text nodes
        - Formatting elements
    end note

    AfterBody --> AfterAfterBody: html closing tag
    AfterBody --> InBody: non-html end tag/content
    note left of AfterBody
        Handles trailing content
        after body
    end note

    AfterAfterBody --> [*]: EOF
    AfterAfterBody --> InBody: unexpected content
    note right of AfterAfterBody
        Final cleanup and
        document completion
    end note
```
