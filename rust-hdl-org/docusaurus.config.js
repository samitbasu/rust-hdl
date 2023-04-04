// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'RustHDL',
  tagline: 'Write FPGA Firmware using Rust! 🎉🦀',
  favicon: 'img/favicon.ico',

  // Set the production url of your site here
  url: 'https://www.rust-hdl.org',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'samitbasu', // Usually your GitHub org/user name.
  projectName: 'rust-hdl', // Usually your repo name.
  trailingSlash: false,

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          routeBasePath: "/",
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          // FIXME
          editUrl:
            'https://github.com/samitbasu/rust-hdl/tree/main/packages/create-docusaurus/templates/shared/',
        },
        blog: false,
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      // Replace with your project's social card
      image: 'img/docusaurus-social-card.jpg',
      navbar: {
        title: 'RustHDL',
        logo: {
          alt: 'CyberFerris Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'tutorialSidebar',
            position: 'left',
            label: 'Tutorial',
          },
          { href: 'https://docs.rs/rust-hdl', label: 'API Docs', position: 'right' },
          {
            href: 'https://crates.io/crates/rust-hdl', label: 'Crates 📦', position: 'right'
          },
          {
            href: 'https://github.com/samitbasu/rust-hdl',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Tutorial',
                to: '/docs/intro',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Discord',
                href: 'https://discord.gg/uESUbfhQub',
              },
            ],
          },
          {
            title: 'More',
            items: [
              { href: 'https://docs.rs/rust-hdl', label: 'API Docs' },
              {
                href: 'https://crates.io/crates/rust-hdl', label: 'Crates 📦'
              },
              {
                label: 'GitHub',
                href: 'https://github.com/samitbasu/rust-hdl',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} RustHDL. Built with Docusaurus.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ['rust', 'verilog']
      },
    }),
};

module.exports = config;
