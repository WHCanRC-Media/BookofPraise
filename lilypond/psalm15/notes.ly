melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f2 f4 f4 g2 bes2 a4 a4 g2 r2 \break

  % Line 2
  g2 a4 bes4 c2 g2 bes4 bes4 a2 g2 r2 \break

  % Line 3
  g2 g4 g4 f2 f2 g4 a4 bes2 r2 \break

  % Line 4
  d2 c4 bes4 a2 g2 bes4 bes4 a2 r2 \break

  % Line 5
  c2 bes4 g4 a2 g2 a4 bes4 g2 f1 \bar "|."
}
