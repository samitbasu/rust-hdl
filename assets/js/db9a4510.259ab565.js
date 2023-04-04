"use strict";(self.webpackChunkrust_hdl_org=self.webpackChunkrust_hdl_org||[]).push([[950],{3905:(e,t,n)=>{n.d(t,{Zo:()=>c,kt:()=>m});var r=n(7294);function a(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function i(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var r=Object.getOwnPropertySymbols(e);t&&(r=r.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,r)}return n}function o(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?i(Object(n),!0).forEach((function(t){a(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):i(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function s(e,t){if(null==e)return{};var n,r,a=function(e,t){if(null==e)return{};var n,r,a={},i=Object.keys(e);for(r=0;r<i.length;r++)n=i[r],t.indexOf(n)>=0||(a[n]=e[n]);return a}(e,t);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(r=0;r<i.length;r++)n=i[r],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(a[n]=e[n])}return a}var l=r.createContext({}),u=function(e){var t=r.useContext(l),n=t;return e&&(n="function"==typeof e?e(t):o(o({},t),e)),n},c=function(e){var t=u(e.components);return r.createElement(l.Provider,{value:t},e.children)},p="mdxType",d={inlineCode:"code",wrapper:function(e){var t=e.children;return r.createElement(r.Fragment,{},t)}},f=r.forwardRef((function(e,t){var n=e.components,a=e.mdxType,i=e.originalType,l=e.parentName,c=s(e,["components","mdxType","originalType","parentName"]),p=u(n),f=a,m=p["".concat(l,".").concat(f)]||p[f]||d[f]||i;return n?r.createElement(m,o(o({ref:t},c),{},{components:n})):r.createElement(m,o({ref:t},c))}));function m(e,t){var n=arguments,a=t&&t.mdxType;if("string"==typeof e||a){var i=n.length,o=new Array(i);o[0]=f;var s={};for(var l in t)hasOwnProperty.call(t,l)&&(s[l]=t[l]);s.originalType=e,s[p]="string"==typeof e?e:a,o[1]=s;for(var u=2;u<i;u++)o[u]=n[u];return r.createElement.apply(null,o)}return r.createElement.apply(null,n)}f.displayName="MDXCreateElement"},2298:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>l,contentTitle:()=>o,default:()=>d,frontMatter:()=>i,metadata:()=>s,toc:()=>u});var r=n(7462),a=(n(7294),n(3905));const i={},o="What is an FPGA anyway? (and who needs one?)",s={unversionedId:"guide/fpga/index",id:"guide/fpga/index",title:"What is an FPGA anyway? (and who needs one?)",description:"That is a great question!  And one I am not really planning to answer.  There are some great",source:"@site/docs/guide/fpga/index.md",sourceDirName:"guide/fpga",slug:"/guide/fpga/",permalink:"/guide/fpga/",draft:!1,editUrl:"https://github.com/samitbasu/rust-hdl/tree/main/packages/create-docusaurus/templates/shared/docs/guide/fpga/index.md",tags:[],version:"current",frontMatter:{},sidebar:"tutorialSidebar",previous:{title:"The Basics",permalink:"/guide/"},next:{title:"Cybersecurity",permalink:"/guide/fpga/cybersecurity"}},l={},u=[],c={toc:u},p="wrapper";function d(e){let{components:t,...n}=e;return(0,a.kt)(p,(0,r.Z)({},c,n,{components:t,mdxType:"MDXLayout"}),(0,a.kt)("h1",{id:"what-is-an-fpga-anyway-and-who-needs-one"},"What is an FPGA anyway? (and who needs one?)"),(0,a.kt)("p",null,"That is a great question!  And one I am not really planning to answer.  There are some great\nresources available if you have never worked with an FPGA before.  The short version is that an\nFPGA can be thought of as collection of digital logic (and analog) circuits that are packaged together\nand that can be reconfigured through software.  This is important to understand, since there is a\nfundamental difference between FPGAs and the CPUs that have become ubiquitous and dominant in the\nelectronics industry.  FPGAs are fundamentally ",(0,a.kt)("em",{parentName:"p"},"massively parallel devices"),".  As such, they are\nnot just really fast processors.  In fact, in terms of raw clock speed, modern CPUs and\n(even some microcontrollers) will run circles around an FPGA.  CPUs are available in ever increasing\nspeeds and with ever more cores and capabilities.  Need a multi-core microcontroller that runs\nat 500Mhz?  No problem.  CPUs clocked at 4GHz?  Tons of high speed DRAM?  Yup.  System-on-a-chip\nthat contains a bunch of nifty peripherals?  Yeah - you can use those too."),(0,a.kt)("p",null,"So why am I starting out with a list of reasons why you don't need an FPGA?  Primarily because FPGAs\nare really good at certain tasks, but to use them effectively is quite difficult.  RustHDL is my effort\nto make them less difficult to use, but it doesn't make the underlying complexity disappear.  You may say\n\"So what do FPGAs do well, then?\".  Great question!  Glad I asked it.  There are a few areas\nwhere FPGAs still hold an advantage over other digital solutions."),(0,a.kt)("ul",null,(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("a",{parentName:"li",href:"/guide/fpga/deterministic-systems"},"Deterministic systems"),"!"),(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("a",{parentName:"li",href:"/guide/fpga/kitchen-analogy"},"True parallelism")),(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("a",{parentName:"li",href:"/guide/fpga/high-speed-io"},"High speed I/O")),(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("a",{parentName:"li",href:"/guide/fpga/low-power"},"Low power designs")),(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("a",{parentName:"li",href:"/guide/fpga/cybersecurity"},"Cybersecurity"))),(0,a.kt)("p",null,"Keep reading for more details!"))}d.isMDXComponent=!0}}]);