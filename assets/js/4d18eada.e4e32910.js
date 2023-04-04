"use strict";(self.webpackChunkrust_hdl_org=self.webpackChunkrust_hdl_org||[]).push([[863],{3905:(e,t,r)=>{r.d(t,{Zo:()=>p,kt:()=>g});var n=r(7294);function i(e,t,r){return t in e?Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}):e[t]=r,e}function a(e,t){var r=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),r.push.apply(r,n)}return r}function s(e){for(var t=1;t<arguments.length;t++){var r=null!=arguments[t]?arguments[t]:{};t%2?a(Object(r),!0).forEach((function(t){i(e,t,r[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(r)):a(Object(r)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(r,t))}))}return e}function l(e,t){if(null==e)return{};var r,n,i=function(e,t){if(null==e)return{};var r,n,i={},a=Object.keys(e);for(n=0;n<a.length;n++)r=a[n],t.indexOf(r)>=0||(i[r]=e[r]);return i}(e,t);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);for(n=0;n<a.length;n++)r=a[n],t.indexOf(r)>=0||Object.prototype.propertyIsEnumerable.call(e,r)&&(i[r]=e[r])}return i}var u=n.createContext({}),o=function(e){var t=n.useContext(u),r=t;return e&&(r="function"==typeof e?e(t):s(s({},t),e)),r},p=function(e){var t=o(e.components);return n.createElement(u.Provider,{value:t},e.children)},d="mdxType",c={inlineCode:"code",wrapper:function(e){var t=e.children;return n.createElement(n.Fragment,{},t)}},h=n.forwardRef((function(e,t){var r=e.components,i=e.mdxType,a=e.originalType,u=e.parentName,p=l(e,["components","mdxType","originalType","parentName"]),d=o(r),h=i,g=d["".concat(u,".").concat(h)]||d[h]||c[h]||a;return r?n.createElement(g,s(s({ref:t},p),{},{components:r})):n.createElement(g,s({ref:t},p))}));function g(e,t){var r=arguments,i=t&&t.mdxType;if("string"==typeof e||i){var a=r.length,s=new Array(a);s[0]=h;var l={};for(var u in t)hasOwnProperty.call(t,u)&&(l[u]=t[u]);l.originalType=e,l[d]="string"==typeof e?e:i,s[1]=l;for(var o=2;o<a;o++)s[o]=r[o];return n.createElement.apply(null,s)}return n.createElement.apply(null,r)}h.displayName="MDXCreateElement"},3485:(e,t,r)=>{r.r(t),r.d(t,{assets:()=>u,contentTitle:()=>s,default:()=>c,frontMatter:()=>a,metadata:()=>l,toc:()=>o});var n=r(7462),i=(r(7294),r(3905));const a={},s="Using RustHDL",l={unversionedId:"guide/rusthdl/index",id:"guide/rusthdl/index",title:"Using RustHDL",description:"This section of the documentation covers the details of using RustHDL to generate firmware.",source:"@site/docs/guide/rusthdl/index.md",sourceDirName:"guide/rusthdl",slug:"/guide/rusthdl/",permalink:"/guide/rusthdl/",draft:!1,editUrl:"https://github.com/samitbasu/rust-hdl/tree/main/packages/create-docusaurus/templates/shared/docs/guide/rusthdl/index.md",tags:[],version:"current",frontMatter:{},sidebar:"tutorialSidebar",previous:{title:"Verilog",permalink:"/guide/programming/verilog"},next:{title:"Representing bits",permalink:"/guide/rusthdl/bits"}},u={},o=[],p={toc:o},d="wrapper";function c(e){let{components:t,...r}=e;return(0,i.kt)(d,(0,n.Z)({},p,r,{components:t,mdxType:"MDXLayout"}),(0,i.kt)("h1",{id:"using-rusthdl"},"Using RustHDL"),(0,i.kt)("p",null,"This section of the documentation covers the details of using RustHDL to generate firmware.  "),(0,i.kt)("p",null,"Here is an overview of the sections."),(0,i.kt)("ul",null,(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/bits"},"bits")," - how to manage arbitrary bit-width signals"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/operators"},"operators")," - operations that are defined on signals"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/signals"},"signals")," - the struct used to represent a signal (wire) in the firmware"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/traits"},"traits")," - the traits used to implement RustHDL"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/synthesizable"},"synthesizable")," - the synthesizable subset of Rust"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/interfaces"},"interfaces")," - how to simplify your designs with interfaces"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/simulation"},"simulation")," - how to simulate your design in RustHDL"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/high_level_synthesis"},"high level synthesis")," - a high level synthesis library written in RustHDL"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/loops"},"loops")," - loops and arrays in RustHDL"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/struct_valued"},"structs")," - using struct-valued signals in RustHDL"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/verilog"},"verilog")," - generating Verilog from your RustHDL code"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("a",{parentName:"li",href:"/guide/rusthdl/wrapping"},"wrapping")," - wrapping black box cores and other Verilog with your RustHDL code")))}c.isMDXComponent=!0}}]);