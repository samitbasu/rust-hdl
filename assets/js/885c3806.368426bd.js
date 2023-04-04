"use strict";(self.webpackChunkrust_hdl_org=self.webpackChunkrust_hdl_org||[]).push([[386],{3905:(e,t,n)=>{n.d(t,{Zo:()=>h,kt:()=>f});var o=n(7294);function a(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function r(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);t&&(o=o.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,o)}return n}function i(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?r(Object(n),!0).forEach((function(t){a(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):r(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function s(e,t){if(null==e)return{};var n,o,a=function(e,t){if(null==e)return{};var n,o,a={},r=Object.keys(e);for(o=0;o<r.length;o++)n=r[o],t.indexOf(n)>=0||(a[n]=e[n]);return a}(e,t);if(Object.getOwnPropertySymbols){var r=Object.getOwnPropertySymbols(e);for(o=0;o<r.length;o++)n=r[o],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(a[n]=e[n])}return a}var l=o.createContext({}),c=function(e){var t=o.useContext(l),n=t;return e&&(n="function"==typeof e?e(t):i(i({},t),e)),n},h=function(e){var t=c(e.components);return o.createElement(l.Provider,{value:t},e.children)},u="mdxType",p={inlineCode:"code",wrapper:function(e){var t=e.children;return o.createElement(o.Fragment,{},t)}},d=o.forwardRef((function(e,t){var n=e.components,a=e.mdxType,r=e.originalType,l=e.parentName,h=s(e,["components","mdxType","originalType","parentName"]),u=c(n),d=a,f=u["".concat(l,".").concat(d)]||u[d]||p[d]||r;return n?o.createElement(f,i(i({ref:t},h),{},{components:n})):o.createElement(f,i({ref:t},h))}));function f(e,t){var n=arguments,a=t&&t.mdxType;if("string"==typeof e||a){var r=n.length,i=new Array(r);i[0]=d;var s={};for(var l in t)hasOwnProperty.call(t,l)&&(s[l]=t[l]);s.originalType=e,s[u]="string"==typeof e?e:a,i[1]=s;for(var c=2;c<r;c++)i[c]=n[c];return o.createElement.apply(null,i)}return o.createElement.apply(null,n)}d.displayName="MDXCreateElement"},1965:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>l,contentTitle:()=>i,default:()=>p,frontMatter:()=>r,metadata:()=>s,toc:()=>c});var o=n(7462),a=(n(7294),n(3905));const r={},i="True Parallelism (The Kitchen Analogy)",s={unversionedId:"guide/fpga/kitchen-analogy",id:"guide/fpga/kitchen-analogy",title:"True Parallelism (The Kitchen Analogy)",description:"CPUs are fast, but they are sequential devices.  Here's another way to think",source:"@site/docs/guide/fpga/kitchen-analogy.md",sourceDirName:"guide/fpga",slug:"/guide/fpga/kitchen-analogy",permalink:"/guide/fpga/kitchen-analogy",draft:!1,editUrl:"https://github.com/samitbasu/rust-hdl/tree/main/packages/create-docusaurus/templates/shared/docs/guide/fpga/kitchen-analogy.md",tags:[],version:"current",frontMatter:{},sidebar:"tutorialSidebar",previous:{title:"High Speed I/O",permalink:"/guide/fpga/high-speed-io"},next:{title:"Saving those Joules",permalink:"/guide/fpga/low-power"}},l={},c=[],h={toc:c},u="wrapper";function p(e){let{components:t,...n}=e;return(0,a.kt)(u,(0,o.Z)({},h,n,{components:t,mdxType:"MDXLayout"}),(0,a.kt)("h1",{id:"true-parallelism-the-kitchen-analogy"},"True Parallelism (The Kitchen Analogy)"),(0,a.kt)("p",null,"CPUs are fast, but they are sequential devices.  Here's another way to think\nabout it.  A CPU is sort of like a person working in a kitchen.  A multi-core CPU\nis sort of like multiple people working in the same kitchen.  When working together\nyou can create some amazing dishes/meals, with each of you focusing on some specialty\nand moving around the kitchen in such a way as to share resources.  If you both need\nthe same knife or pot, you need to wait for someone to finish.  If you are sharing\nthe preparation of a dish, you need to hand it off at the right moment.  You may have\nmany tasks running in parallel (oven, mixer, blender, etc), but you can only focus\non one at a time.  And the resources are typically shared between the various people\nin the kitchen."),(0,a.kt)("p",null,"An FPGA, on the other hand, is more like a food factory.  Imagine you need to crank out\nthousands of identical (or very similar) dishes, with a high degree of predictability\n(determinism).  The most efficient way to do that is to set up a large parallel\nassembly-line like operation, in which each station does only one task, and throughput\nis achieved by increasing the number of stations doing that task.  That means\nthat each station has a dedicated set of resources (e.g, a cutting board, knife, onions),\nbut that you may have many of those stations operating in parallel to increase\nthe number of onions chopped per hour."),(0,a.kt)("p",null,"The second part of that food factory analogy is the interconnect.  If you have\never seen the inside of a food factory, it has equipment meant to connect the different\nstations together.  Conveyor belts, usually, but it could be vehicles, robots,\netc.  These move partial products from one part of the factory to another.  "),(0,a.kt)("p",null,"Now, the reason I like the factory/kitchen analogy is that it fits on a bunch\nof different levels.  A factory is good for producing at scale, identical (and often simplified)\nrecipies.  A kitchen excels at flexibility, improvisation, and adaptation.  Did you\nplan on having a block of Tofu, only to find someone else already used it?  Find a\nsubstitute!  Or stop and go get some.  Or make it.  Or bother a neighbor.  You get the\nidea."),(0,a.kt)("p",null,"Factories, on the other hand, do not deal well with shortages.  How do you adapt an\nonion station when there are no onions?  "),(0,a.kt)("p",null,"This tradeoff between flexibility, and adaptability and the ability to scale is key\nto why FPGAs are still relevant.  They have fewer applications these days than they\nused to, but there are still plenty of applications where an FPGA can elegantly and\nefficiently solve a problem that is very hard to do on a microprocessor.  Incidentally,\nyou can, of course, build microprocessors on FPGAs, and some FPGAs come with CPUs\nbuilt in.  I won't focus on either of those at this point.  A microprocessor on an\nFPGA is an excellent way to bridge the gap between the two technologies, but requires\nsome fairly advanced techniques that we won't start with."))}p.isMDXComponent=!0}}]);