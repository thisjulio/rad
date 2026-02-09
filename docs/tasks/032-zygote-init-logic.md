# Task 032: Zygote-light Init Logic

## Descrição Detalhada
No Android real, o Zygote pré-carrega todas as classes Java. No nosso runner, faremos um "Zygote-light" que inicializa a ART e carrega o mínimo para rodar a Activity do App.

## Detalhes de Implementação (Rust)
1.  Configurar variáveis de ambiente críticas:
    - `ANDROID_ROOT`, `ANDROID_DATA`, `ANDROID_RUNTIME_ROOT`.
    - `LD_LIBRARY_PATH` apontando para as pastas de libs do payload.
2.  Executar o binário `app_process` (ou equivalente) passando os parâmetros de classe principal.
3.  Isolamento: Garantir que o processo ART rode dentro do sandbox criado na Task 021.
