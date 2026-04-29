melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c'2 c4 c4 d4 d4 c2 \break

  % Line 2
  c4 bes4 a4 g4 a4 g2 f2 r2 \break

  % Line 3
  c'2 c4 c4 d4 d4 c2 \break

  % Line 4
  c4 bes4 a4 g4 a4 g2 f2 r2 \break

  % Line 5
  g2 g4 g4 a4 c4 g2 \break

  % Line 6
  g4 a4 c4 d4 bes4 c2 \break

  % Line 7
  c4 d4 c4 bes4 a4 bes2 \break

  % Line 8
  a2 g4 f4 f4 e4 f2 \bar "|."
}
