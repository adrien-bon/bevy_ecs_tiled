// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded affix "><a href="FAQ.html">FAQ</a></li><li class="chapter-item expanded affix "><li class="part-title">Design &amp; Explanation</li><li class="chapter-item expanded "><a href="design/why_tiled.html"><strong aria-hidden="true">1.</strong> Why use Tiled ?</a></li><li class="chapter-item expanded "><a href="design/entities_hierarchy.html"><strong aria-hidden="true">2.</strong> Entities hierarchy and marker components</a></li><li class="chapter-item expanded "><a href="design/z_order.html"><strong aria-hidden="true">3.</strong> Z-ordering</a></li><li class="chapter-item expanded "><a href="design/map_events.html"><strong aria-hidden="true">4.</strong> Map loading events</a></li><li class="chapter-item expanded "><div><strong aria-hidden="true">5.</strong> Coordinates conversion between Tiled and Bevy</div></li><li class="chapter-item expanded affix "><li class="part-title">How-To Guides</li><li class="chapter-item expanded "><a href="guides/getting-started.html"><strong aria-hidden="true">6.</strong> Getting started</a></li><li class="chapter-item expanded "><a href="guides/spawn_reload.html"><strong aria-hidden="true">7.</strong> Spawn / Despawn / Reload a map</a></li><li class="chapter-item expanded "><a href="guides/physics.html"><strong aria-hidden="true">8.</strong> Add physics colliders</a></li><li class="chapter-item expanded "><a href="guides/properties.html"><strong aria-hidden="true">9.</strong> Using Tiled custom properties</a></li><li class="chapter-item expanded "><a href="guides/debug.html"><strong aria-hidden="true">10.</strong> Debug your project</a></li><li class="chapter-item expanded affix "><li class="part-title">Migration Guides</li><li class="chapter-item expanded "><a href="migrations/v0_11.html"><strong aria-hidden="true">11.</strong> Migration to v0.11</a></li><li class="chapter-item expanded "><a href="migrations/v0_10.html"><strong aria-hidden="true">12.</strong> Migration to v0.10</a></li><li class="chapter-item expanded "><a href="migrations/v0_9.html"><strong aria-hidden="true">13.</strong> Migration to v0.9</a></li><li class="chapter-item expanded "><a href="migrations/v0_8.html"><strong aria-hidden="true">14.</strong> Migration to v0.8</a></li><li class="chapter-item expanded "><a href="migrations/v0_7.html"><strong aria-hidden="true">15.</strong> From v0.6.X to v0.7.X</a></li><li class="chapter-item expanded "><a href="migrations/v0_6.html"><strong aria-hidden="true">16.</strong> From v0.5.X to v0.6.X</a></li><li class="chapter-item expanded "><a href="migrations/v0_5.html"><strong aria-hidden="true">17.</strong> From v0.4.X to v0.5.X</a></li><li class="chapter-item expanded "><a href="migrations/v0_4.html"><strong aria-hidden="true">18.</strong> From v0.3.X to v0.4.X</a></li><li class="chapter-item expanded affix "><li class="part-title">Miscellaneous</li><li class="chapter-item expanded "><a href="misc/useful-links.html"><strong aria-hidden="true">19.</strong> Useful links</a></li><li class="chapter-item expanded "><a href="misc/contributing.html"><strong aria-hidden="true">20.</strong> Contributing</a></li><li class="chapter-item expanded "><a href="misc/api-reference.html"><strong aria-hidden="true">21.</strong> API reference</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
