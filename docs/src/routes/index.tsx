import { Title } from "@solidjs/meta";
import { FeatureCard } from "~/components/FeatureCard";

const FEATURES = [
  {
    emoji: "⚡️",
    title: "Fast",
    description: "High-performance formatter implemented in Rust",
  },
  {
    emoji: "🎯",
    title: "Accurate",
    description: "Full compliance with TOML specification",
  },
  {
    emoji: "🛠",
    title: "Customizable",
    description: "Flexible configuration for your project needs",
  },
] as const;

export default function Home() {
  return (
    <div>
      <Title>Tombi - Modern TOML Formatter</Title>

      <section class="text-center mb-24">
        <div class="max-w-6xl mx-auto px-4">
          <h1 class="sr-only">Tombi</h1>
          <div class="relative mb-16 w-screen -mx-[calc((100vw-100%)/2)] overflow-hidden bg-gradient-to-b from-gray via-tombi-1000/10 to-gray dark:from-gray-900 dark:via-tombi-900/10 dark:to-gray-900">
            <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(0,0,102,0.05),transparent_70%)] dark:bg-[radial-gradient(circle_at_center,rgba(255,255,255,0.03),transparent_70%)]" />
              <svg width="100%" height="100%" viewBox="0 0 800 200" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="fill-rule:evenodd;clip-rule:evenodd;stroke-linejoin:round;stroke-miterlimit:2;">
                  <g transform="matrix(1.00856,0,0,1.00856,131.954,-3.20241)">
                      <g transform="matrix(10.3281,0,0,10.5452,407.206,154.298)">
                          <text x="0px" y="0px" style="font-family:'Helvetica-Bold', 'Helvetica';font-weight:700;font-size:12px;fill:rgb(156,66,33);">oi</text>
                      </g>
                      <g transform="matrix(10.7472,0,0,10.5452,209.019,154.358)">
                          <text x="0px" y="0px" style="font-family:'Helvetica-Bold', 'Helvetica';font-weight:700;font-size:12px;fill:rgb(255,253,232);">oml</text>
                      </g>
                      <g transform="matrix(10.7472,0,0,10.5452,139.754,154.358)">
                          <text x="0px" y="0px" style="font-family:'Helvetica-Bold', 'Helvetica';font-weight:700;font-size:12px;fill:rgb(255,253,232);">T</text>
                      </g>
                  </g>
                  <g transform="matrix(2.20012,0,0,2.20012,142.569,23.4277)">
                      <g transform="matrix(4.15375,0,0,4.9461,-21.1003,53.6106)">
                          <path d="M12.299,1.084L12.299,-7.523L10.939,-7.523L10.939,-8.719L13.881,-8.719L13.881,2.309L10.939,2.309L10.939,1.084L12.299,1.084Z" style="fill:rgb(156,66,33);fill-rule:nonzero;"/>
                          <path d="M3.691,-8.719L3.691,-7.5L2.332,-7.5L2.332,1.084L3.691,1.084L3.691,2.309L0.75,2.309L0.75,-8.719L3.691,-8.719Z" style="fill:rgb(156,66,33);fill-rule:nonzero;"/>
                      </g>
                      <g transform="matrix(0.0911527,0,0,0.0976489,-39.1848,-12.2739)">
                          <path d="M524.898,793.744C522.809,783.244 523.687,773.083 524.303,762.958C524.761,755.434 525.737,747.942 526.513,740.123C524.155,741.133 512.485,762.295 504.072,768.641C499.287,772.251 477.066,781.902 473.159,781.646C469.494,781.405 471.924,774.612 473.053,770.757C476.675,758.392 479.788,746.036 484.253,734.006C485.631,730.295 487.227,726.608 487.211,721.925C483.62,724.906 445.991,749.614 434.892,751.739C433.021,752.097 430.958,751.303 428.982,751.291C425.495,751.27 425.045,749.147 425.843,746.594C430.879,730.479 449.89,692.263 449.572,687.433C442.363,695.151 375.923,683.208 374.972,679.435C373.985,675.522 378.31,673.698 381.332,672.148C408.052,658.443 433.607,642.757 458.698,626.334C474.21,616.181 492.819,607.411 501.438,591.047C505.208,583.89 504.315,580.446 502.946,576.591C465.3,598.576 363.219,545.655 358.256,543.488C352.353,540.911 294.562,514.701 249.714,493.055C238.088,487.444 272.389,484.969 272.026,483.066C271.723,481.476 201.144,472.107 186.208,465.751C182.003,463.962 184.664,460.354 186.423,458.594C190.18,454.835 226.704,441.542 223.001,441.321C215.362,440.867 89.92,435.172 105.652,420.219C113.129,413.111 191.89,402.569 188.582,402.453C175.085,401.981 60.262,368.003 68.441,355.155C72.694,348.473 181.659,363.557 178.257,361.329C168.87,355.18 94.258,262.096 114.76,268.065C126.166,271.385 263.821,373.557 374.383,413.842C389.178,419.232 462.36,444.131 477.712,447.252C482.492,448.223 485.77,447.52 489.162,443.09C497.558,432.127 509.923,428.637 523.131,427.776C534.559,427.03 545.073,429.967 555.299,435.184C567.18,441.246 579.362,442.69 592.751,438.248C615.735,430.623 635.066,417.287 653.662,402.6C781.851,301.351 808.633,175.194 813.663,173.037C818.229,177.079 820.544,228.917 814.489,260.007C845.634,242.243 861.939,142.417 868.824,142.322C877.819,142.199 884.814,226.354 861.806,258.807C918.423,231.313 940.146,168.995 945.986,185.267C950.929,199.041 948.908,207.801 937.559,232.281C921.54,266.833 919.117,263.302 897.291,294.936C933.023,297.545 963.354,268.366 965.537,281.608C967.136,291.299 906.754,336.744 896.354,339.488C915.201,347.772 945.614,334.497 952.028,336.9C978.279,346.738 828.343,439.37 832.112,439C845.127,437.721 859.845,431.753 857.918,437.162C855.817,443.061 730.724,525.523 703.047,546.937C692.023,555.466 658.249,579.028 614.317,572.138C613.085,574.046 617.433,587.769 628.264,595.825C658.575,618.371 683.512,627.973 719.249,640.211C728.191,643.273 763.228,652.677 764.106,656.146C766.252,664.618 736.131,684.362 693.207,673.608C693.896,676.503 699.599,682.057 701.264,683.593C712.258,693.733 721.287,705.427 728.894,718.256C732.136,723.723 731.601,724.37 725.363,724.874C704.916,726.526 670.695,701.958 668.16,703.884C665.531,705.882 685.162,740.213 690.605,757.081C691.572,760.078 693.115,763.957 691.618,766.046C689.565,768.914 649.297,747.118 638.104,730.716C637.124,729.279 636.933,727.145 634.713,726.705C632.39,728.138 633.775,729.935 634.238,731.557C635.927,737.48 642.203,776.101 643.323,784.003C643.885,787.965 642.088,788.816 638.35,788.185C627.101,786.286 619.982,778.621 613.101,770.597C608.63,765.382 605.169,759.497 602.243,753.288C601.498,751.705 601.091,749.834 599.056,748.702C597.324,752.085 598.304,755.448 598.264,758.641C598.082,773.092 593.064,785.752 584.494,797.176C582.729,799.529 580.961,798.447 579.431,797.032C575.606,793.496 572.568,789.336 570.169,784.708C567.74,780.022 565.287,775.348 562.804,770.591C557.948,774.139 557.565,779.745 554.72,783.907C551.38,788.794 532.387,808.807 529.635,805.167C527.181,801.923 526.401,797.888 524.898,793.744" style="fill:rgb(255,253,232);fill-rule:nonzero;"/>
                      </g>
                  </g>
              </svg>
          </div>

          <p class="text-xl text-tombi-800/80 dark:text-gray-300 mb-16 max-w-2xl mx-auto">
            Next Generation TOML Formatter - Bringing elegance and precision to your TOML configurations
          </p>

          <div class="grid md:grid-cols-3 gap-8 mb-16">
            {FEATURES.map((feature) => (
              <FeatureCard {...feature} />
            ))}
          </div>

          <div class="flex gap-4 justify-center">
            <a
              href="/documentation/getting-started/installation"
              class="px-8 py-4 bg-tombi-900 text-white rounded-xl hover:bg-tombi-800 transition-colors shadow-lg hover:shadow-xl"
            >
              Get Started
            </a>
            <a
              href="/documentation"
              class="px-8 py-4 bg-white dark:bg-tombi-900/30 border border-tombi-200 dark:border-tombi-700 rounded-xl hover:bg-tombi-50 dark:hover:bg-tombi-900/50 transition-colors text-tombi-900 dark:text-white shadow-lg hover:shadow-xl"
            >
              View Docs
            </a>
          </div>
        </div>
      </section>

      <section class="max-w-3xl mx-auto px-4">
        <h2 class="text-3xl font-bold text-center mb-8 bg-gradient-to-r from-tombi-900 to-tombi-700 dark:from-white dark:to-tombi-200 bg-clip-text text-transparent">
          Simple and Easy to Use
        </h2>
        <pre class="p-8 bg-tombi-900 text-white rounded-xl overflow-x-auto shadow-lg text-left">
          <code class="text-left">{`# Before
title="TOML Example"
[package]
name="my-project"
version="0.1.0"
authors=["John Doe <john@example.com>",]

# After
title = "TOML Example"

[package]
name = "my-project"
version = "0.1.0"
authors = [
  "John Doe <john@example.com>",
]`}</code>
        </pre>
      </section>
    </div>
  );
}
