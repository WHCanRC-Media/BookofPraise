melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f2 d2 g2 f4 f4 g4 a4 bes4 a4 g2 f2 r2 \break

  % Line 2
  bes2 c4 c4 d2 c2 a4 bes4 g4 g4 f2 r2 \break

  % Line 3
  f2 g2 a2 f4 f4 g4 bes4 bes4 a4 bes2 r2 \break

  % Line 4
  bes2 g4 f4 bes2 a2 f4 g4 a4 bes4 g2 f1 \bar "|."
}
