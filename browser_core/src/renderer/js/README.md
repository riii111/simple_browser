## 概要

JavaScriptエンジンを実装する。

## 実装内容

JavaScriptのコードを解析し、実行する。

- JavaScriptエンジンにはインタプリタ、コンパイラ、JITコンパイラ（Just In Time Compiler）などの実装があるが、ここではインタプリタのみを実装する。
- CSR（Client Side Rendering）、かつ以下の簡易機能のみを実装。
  - 足し算・引き算
  - 変数の定義と使用
  - 関数の定義と呼び出し
