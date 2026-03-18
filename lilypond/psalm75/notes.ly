melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c'2 bes2 a4 d4 c2 bes2 a4 \break

  % Line 2
  d4 c2 bes2 a2 g4 g4 f2 r2 \break

  % Line 3
  f2 a2 g4 c4 c4 bes4 c2 \break

  % Line 4
  d4 d4 c4 a4 bes4 bes4 a2 r2 \break

  % Line 5
  c2 bes2 a4 c4 bes4 a4 g2 r2 \break

  % Line 6
  a2 c2 f,4 bes4 a2 g2 f1 \bar "|."
}
