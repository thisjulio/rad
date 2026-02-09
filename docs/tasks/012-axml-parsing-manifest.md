# Task 012: Android Binary XML (AXML) Manifest Parsing

## Descrição Detalhada
O arquivo `AndroidManifest.xml` dentro de um APK não é um XML comum; ele é compilado em um formato binário chamado AXML para economia de espaço e performance no Android. Precisamos extrair informações críticas dele (Package Name, Version e Main Activity).

## Detalhes de Implementação (Rust)
1.  Extrair os bytes de `AndroidManifest.xml` do ZIP.
2.  Implementar ou utilizar um parser de AXML (ex: `apk-info` ou `rusty-axml`).
3.  **Dados a extrair**:
    - Atributo `package` da tag `<manifest>`.
    - `android:name` da `<activity>` que possui o intent filter `android.intent.action.MAIN`.

## Por que não usar XML comum?
O Android Runtime não lê XML de texto. Tentar ler como texto resultará em lixo binário. O parser deve entender o String Pool e o Resource Map do formato AXML.

## Referências
- [Apktool (Referência de formato)](https://github.com/iBotPeaches/Apktool)
- [Crate `apk-info`](https://docs.rs/apk-info/latest/apk_info/)
