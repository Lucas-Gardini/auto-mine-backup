<div align="center">
  <img src="./app-icon.png" width="96" alt="Logo" />
  <h1>Auto Mine Backup</h1>
  <p><b>Backup automático de mundos Minecraft, com interface moderna e integração nativa!</b></p>
  <p>
    <img src="https://img.shields.io/badge/tauri-vite-blue?logo=tauri" />
    <img src="https://img.shields.io/badge/vue-3.x-green?logo=vue.js" />
    <img src="https://img.shields.io/badge/typescript-4.x-blue?logo=typescript" />
    <img src="https://img.shields.io/badge/rust-stable-orange?logo=rust" />
  </p>
</div>

---

## ✨ Sobre o projeto

O **Auto Mine Backup** é um aplicativo desktop windows que realiza backups automáticos dos seus mundos do Minecraft, detectando quando você está jogando e salvando cópias de segurança de forma inteligente. Desenvolvido com Tauri, Vue 3 e Rust, oferece performance nativa, interface moderna e integração total com o sistema operacional.

---

## 🚀 Funcionalidades

-   Backup automático dos mundos do Minecraft
-   Detecção inteligente do status do jogo
-   Limite de backups configurável
-   Interface gráfica moderna (Vue 3 + Vite)
-   Ícone na bandeja do sistema (tray)
-   Logs detalhados e gerenciamento automático do arquivo de log
-   Configuração de destino dos backups

---

## ⚡ Instalação e uso

### Pré-requisitos

-   [Node.js](https://nodejs.org/) (recomendado: v18+)
-   [Rust](https://www.rust-lang.org/tools/install)
-   [Bun](https://bun.sh/) (opcional, se usar Bun)

### Instalação

```bash
# Clone o repositório
$ git clone https://github.com/seu-usuario/auto-mine-backup.git
$ cd auto-mine-backup

# Instale as dependências do frontend
$ bun install # ou npm install

# Instale as dependências do Tauri
$ cargo build
```

### Execução

```bash
# Inicie o frontend
$ bun run dev # ou npm run dev

# Em outro terminal, rode o Tauri
$ cargo tauri dev
```

---

## 🛠️ Tecnologias

-   [Tauri](https://tauri.app/) — Shell nativo multiplataforma
-   [Vue 3](https://vuejs.org/) — Interface reativa
-   [TypeScript](https://www.typescriptlang.org/) — Tipagem estática
-   [Rust](https://www.rust-lang.org/) — Backend nativo
-   [Vite](https://vitejs.dev/) — Bundler ultrarrápido

---

## ⚙️ Configuração

-   Configure o destino dos backups e o número máximo de backups pela interface.
-   O app detecta automaticamente o mundo ativo do Minecraft e realiza backups periódicos.
-   Os logs são gerenciados automaticamente, evitando crescimento excessivo do arquivo.

---

## ❓ FAQ

**O app funciona com Minecraft pirata?**

> Sim, desde que a estrutura de pastas seja compatível.

**Posso restaurar um backup?**

> Basta copiar o arquivo ZIP gerado para a pasta `saves` do Minecraft.

**O app consome muitos recursos?**

> Não, o monitoramento é leve e o backup é feito em segundo plano.

---

## 👨‍💻 Contribuição

1. Faça um fork do projeto
2. Crie sua branch: `git checkout -b minha-feature`
3. Commit suas alterações: `git commit -m 'feat: minha nova feature'`
4. Push para o fork: `git push origin minha-feature`
5. Abra um Pull Request

---

## 📄 Licença

Este projeto está sob a licença MIT.

---

## 💡 Créditos

Desenvolvido por [Lucas Gardini](https://github.com/Lucas-Gardini).

---

<div align="center">
  <sub>Feito com ❤️ usando Tauri, Vue, TypeScript e Rust</sub>
</div>
