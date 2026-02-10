# Task 012: Android Binary XML (AXML) Manifest Parsing

## Descrição Detalhada
O arquivo `AndroidManifest.xml` dentro de um APK não é um XML comum; ele é compilado em um formato binário chamado AXML para economia de espaço e performance no Android. Precisamos extrair informações críticas dele (Package Name, Version e Main Activity).

## Fluxo TDD
- [x] **Red**: Criar um teste que recebe bytes de um manifesto binário real (ou simplificado) e tenta extrair o package name, falhando inicialmente.
- [x] **Green**: Implementar a integração com o parser AXML (ex: `apk-info`).
- [x] **Refactor**: Encapsular os dados extraídos em uma struct `AppManifest`.

## Detalhes de Implementação (Rust)
1.  Extrair os bytes de `AndroidManifest.xml` do ZIP.
2.  Implementar ou utilizar um parser de AXML (ex: `apk-info` ou `rusty-axml`).
3.  **Dados a extrair**:
    - Atributo `package` da tag `<manifest>`.
    - `android:name` da `<activity>` que possui o intent filter `android.intent.action.MAIN`.

## Referências
- [Apktool (Referência de formato)](https://github.com/iBotPeaches/Apktool)
- [Crate `apk-info`](https://docs.rs/apk-info/latest/apk_info/)
