<div align="center">
<img alt="balsa logo" src="/modules/branding/balsa-horiz.png" width="33%"/>
<p>Lightweight, malleable Linux.</p>
<h3 style="color:red">This repository is incomplete!</h3>
</div>
<hr>
<p>Balsa Linux is a new distribution, powered by Nix. It allows for out-of-the-box customisation, is jam-packed with all the latest features, and is designed to be easy to use and modify.</p>
<div align="center"><h2>Requirements:</h2></div>
<ul>
<li>64-bit CPU (x86_64 or aarch64) [IMPORTANT! aarch64 will be missing features compared to the x86_64 version. They will not be included in the aarch64 installer.]</li>
<li>2-4GB RAM required | 8+GB RAM recommended</li>
<li>20GB+ of free disk space required | 60GB+ recommended</li>
<li>An active network connection</li>
</ul>
<div align="center"><sub>These requirements are based on running a desktop environment, not a minimal install.</sub></div>
<div align="center"><h2>Default Software:</h2></div>
<ul>
<li><strong>LibreWolf</strong>: Browse the web privately, using a hardened fork of Firefox.</li>
<li><strong>Neovim + LazyVim</strong>: A powerful terminal-based text editor that is focused on productivity.</li>
<li><strong>Micro:</strong> A lightweight terminal-based editor, designed to be a continuation of the GNU Nano text editor. By default, uses normal keybinds that you would expect from a normal text editor.</li>
<li><strong>Zed:</strong> A GPU-accelerated text editor application, similar to VSCode.</li>
</ul>
<p>These above have been chosen as Balsa's default software for their ease of use, flexibility, and reliability.</p>
<div align="center"><sub>Used to the commands of <code>vim</code> and <code>nano?</code> Don't worry, those commands route to the new editors, meaning your workflow should not be disrupted.</sub></div>
<div align="center"><h2>Features:</h2></div>
<ul>
<li><strong>zsh:</strong> A powerful, feature-rich shell that is designed to be a drop-in replacement for Bash. Powered by OMZ, you get autosuggestions, syntax highlighting, and more. OMZ also allows for in-depth customisation of your terminal experience.</li>
<li><strong>OpenZFS Support:</strong> Balsa Linux supports OpenZFS out-of-the-box, providing advanced storage management capabilities.</li>
<li><strong>Kernel options:</strong> Why stick with mainline or LTS? Balsa Linux offers the Zen and Xanmod kernels, providing improved performance and stability.</li>
<li><strong>Nix package manager:</strong> Balsa Linux comes with the Nix package manager, allowing for easy and reproducible package management. Provided alongside the package manager are utilities to streamline the Nix experience of installing, managing, and reproducing.</li>
<li><strong>balsa-pkg:</strong> Finding nixpkgs has never been easier with this fzf-powered TUI tool. Simply search for a package that you want to install!</li>
<li><strong>Tuning profiles:</strong> Balsa Linux offers tuning profiles to optimize system performance and stability, depending on your workload. You can run the gaming profile for games, development profile for writing code, and so much more.</li>
</ul>

<div align="center"><h2>Planned:</h2></div>
<ul>
<li>CachyOS kernel support for increased performance</li>
<li>Offer browser choice during install between LibreWolf, Helium, Zen, Firefox, and Chromium.</li>
<li>Media creation tool to build ISOs and create installation media.</li>
<li>H-Balsa, a hardened set of configs, utils, and modules for Balsa to achieve an OpenBSD or greater level of security.</li>
</ul>

<div align="center"><h2>Credits:</h2></div>
<ul>
<li>Lead Developer: @aylah-a63</li>
<li>Contributors: <br>Want to see your name here? Feel free to contribute!</li>
<li>Developers of Nix, nixpkgs, NixOS, all preloaded software</li>
<li>You, for installing!</li>
</ul>
