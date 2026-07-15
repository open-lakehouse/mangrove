import type { SVGProps } from "react";

// Official Unity Catalog mark, vendored from the LF AI & Data artwork repo
// (projects/unity_catalog/icon/black/SVG/uc_icon_black.svg) and made monochrome:
// the paths carry no fill, so they inherit `fill="currentColor"` from the root
// and recolor with the surrounding text color. Trademark usage:
// https://www.linuxfoundation.org/trademark-usage
export function UnityCatalogIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 639 650.5"
      fill="currentColor"
      role="img"
      aria-label="Unity Catalog"
      {...props}
    >
      <path d="M317.7,215.8l-94.8,54.7V380l94.8,54.7l94.8-54.7V270.5L317.7,215.8z" />
      <path d="M517.6,209.8h109l-54.5-94.4L517.6,209.8z" />
      <path d="M517.6,440.6l54.6,94.5l54.5-94.5H517.6z" />
      <path d="M263.1,650.5h109L317.7,556L263.1,650.5z" />
      <path d="M8.7,440.6l54.6,94.5l54.6-94.5H8.7z" />
      <path d="M8.7,209.8h109.1l-54.6-94.4L8.7,209.8z" />
      <path d="M263.1,0l54.6,94.5L372.2,0H263.1z" />
      <path d="M421.8,18.4L358.5,128l109.6,63.3l63.3-109.6L421.8,18.4z" />
      <path d="M639,260.7H512.4v126.6H639V260.7z" />
      <path d="M468.1,459l-109.6,63.3l63.3,109.6l109.6-63.3L468.1,459z" />
      <path d="M167.3,459.1L104,568.7L213.6,632l63.3-109.6L167.3,459.1z" />
      <path d="M126.6,262H0v126.6h126.6V262z" />
      <path d="M213.6,18.5L104,81.8l63.3,109.6l109.6-63.3L213.6,18.5z" />
    </svg>
  );
}
