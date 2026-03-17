melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c'2 g4 g4 c4 c4 bes4 bes4 a2 g2 r2 \break

  % Line 2
  g2 a4 c4 bes4 g4 bes4 bes4 a2 g2 r2 \break

  % Line 3
  ees2 f4 aes4 g4 ees4 g2 f2 ees2 r2 \break

  % Line 4
  g2 ees4 ees4 c4 ees4 f2 d2 c2 r2 \bar "|."
}
