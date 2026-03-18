melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c'2 a4 bes4 c2 c2 a4 bes4 g4 g4 f2 r2 \break

  % Line 2
  f2 a4 bes4 c2 bes2 a4 g4 a4 c4 bes2 c2 r2 \break

  % Line 3
  c2 bes4 a4 g2 c2 bes4 a4 bes4 c4 d2 c2 r2 \break

  % Line 4
  g2 a4 bes4 c2 g2 a4 bes4 g4 g4 f1 \bar "|."
}
